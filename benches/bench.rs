#![feature(test)]

extern crate blake2b_simd;
extern crate test;

use blake2b_simd::*;
use test::Bencher;

#[bench]
fn blake2b_avx2_one_block(b: &mut Bencher) {
    let input = &[0; BLOCKBYTES];
    b.bytes = input.len() as u64;
    b.iter(|| blake2b(input));
}

#[bench]
fn blake2b_avx2_one_megabyte(b: &mut Bencher) {
    let input = &[0; 1_000_000];
    b.bytes = input.len() as u64;
    b.iter(|| blake2b(input));
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn blake2b_avx2_compress(b: &mut Bencher) {
    if !is_x86_feature_detected!("avx2") {
        return;
    }
    let input = &[0; BLOCKBYTES];
    b.bytes = input.len() as u64;
    let mut h = [0; 8];
    b.iter(|| unsafe { benchmarks::compress_avx2(&mut h, input, 0, 0, 0) });
}

#[bench]
fn blake2b_portable_one_block(b: &mut Bencher) {
    let input = &[0; BLOCKBYTES];
    b.bytes = input.len() as u64;
    b.iter(|| {
        let mut state = State::new();
        benchmarks::force_portable(&mut state);
        state.update(input);
        state.finalize()
    });
}

#[bench]
fn blake2b_portable_one_megabyte(b: &mut Bencher) {
    let input = &[0; 1_000_000];
    b.bytes = input.len() as u64;
    b.iter(|| {
        let mut state = State::new();
        benchmarks::force_portable(&mut state);
        state.update(input);
        state.finalize()
    });
}

#[bench]
fn blake2b_portable_compress(b: &mut Bencher) {
    let input = &[0; BLOCKBYTES];
    b.bytes = input.len() as u64;
    let mut h = [0; 8];
    b.iter(|| benchmarks::compress_portable(&mut h, input, 0, 0, 0));
}
