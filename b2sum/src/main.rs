extern crate blake2b_simd;
extern crate memmap;
extern crate os_pipe;
#[macro_use]
extern crate structopt;

use blake2b_simd::{Hash, Params, State};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::exit;
use structopt::StructOpt;

#[cfg(test)]
mod test;
#[cfg(test)]
#[macro_use]
extern crate duct;
#[cfg(test)]
extern crate tempfile;

#[derive(Debug, StructOpt)]
#[structopt(author = "")]
struct Opt {
    #[structopt(parse(from_os_str), default_value = "-")]
    /// Any number of filepaths, or - for standard input.
    input: Vec<PathBuf>,

    #[structopt(short = "l", long = "length", default_value = "512")]
    /// The size of the output in bits. Must be a multiple of 8.
    length_bits: usize,

    #[structopt(long = "mmap")]
    /// Read input with memory mapping.
    mmap: bool,
}

enum Input {
    Stdin,
    File(File),
    Mmap(memmap::Mmap),
}

fn open_input(path: &Path, mmap: bool) -> io::Result<Input> {
    Ok(if path == Path::new("-") {
        if mmap {
            let stdin_file = os_pipe::dup_stdin()?.into();
            Input::Mmap(unsafe { memmap::Mmap::map(&stdin_file)? })
        } else {
            Input::Stdin
        }
    } else {
        let file = File::open(path)?;
        if mmap {
            Input::Mmap(unsafe { memmap::Mmap::map(&file)? })
        } else {
            Input::File(file)
        }
    })
}

fn hash_one(input: Input, hash_length: usize) -> io::Result<Hash> {
    let mut state = Params::new().hash_length(hash_length).to_state();
    match input {
        Input::Stdin => {
            let stdin = io::stdin();
            let mut stdin = stdin.lock();
            read_write_all(&mut stdin, &mut state)?;
        }
        Input::File(mut file) => {
            read_write_all(&mut file, &mut state)?;
        }
        Input::Mmap(mmap) => {
            state.update(&mmap);
        }
    }
    Ok(state.finalize())
}

fn read_write_all<R: Read>(reader: &mut R, writer: &mut State) -> io::Result<()> {
    // Why 32728 (2^15)? Basically, that's just what coreutils uses. When I benchmark lots of
    // different sizes, a 4 MiB heap buffer actually seems to be the best size, possibly 8% faster
    // than this. Though repeatedly hashing a gigabyte of random data might not reflect real world
    // usage, who knows. At the end of the day, when we really care about speed, we're going to use
    // --mmap and skip buffering entirely. The main goal of this program is to compare the
    // underlying hash implementations (which is to say OpenSSL, which coreutils links against),
    // and to get an honest comparison we might as well use the same buffer size.
    let mut buf = [0; 32768];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            return Ok(());
        }
        writer.write_all(&buf[..n])?;
    }
}

fn do_path(path: &Path, opt: &Opt) -> io::Result<Hash> {
    let input = open_input(path, opt.mmap)?;
    let hash_length = opt.length_bits / 8;
    hash_one(input, hash_length)
}

fn main() {
    let opt = Opt::from_args();

    if opt.length_bits == 0 || opt.length_bits > 512 || opt.length_bits % 8 != 0 {
        eprintln!("Invalid length.");
        exit(1);
    }

    let mut did_error = false;
    for path in &opt.input {
        let path_str = path.to_string_lossy();
        match do_path(path, &opt) {
            Ok(hash) => println!("{}  {}", hash.to_hex(), path_str),
            Err(e) => {
                did_error = true;
                eprintln!("b2sum: {}: {}", path_str, e);
            }
        }
    }
    if did_error {
        exit(1);
    }
}
