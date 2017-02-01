#![feature(test)]

extern crate test;
extern crate bitsparrow;
use bitsparrow::{Encoder, Decoder};

use test::Bencher;

#[bench]
fn allocate_8(b: &mut Bencher) {
    b.iter(|| {
        let foo: Vec<u8> = Vec::with_capacity(8);

        foo
    })
}

#[bench]
fn encode_u64(b: &mut Bencher) {
    b.iter(|| {
        Encoder::encode(::std::u64::MAX)
    })
}

#[bench]
fn encode_f64(b: &mut Bencher) {
    b.iter(|| {
        Encoder::encode(3.141592653589793f64)
    })
}

#[bench]
fn encode_slice(b: &mut Bencher) {
    b.iter(|| {
        Encoder::encode(b"hello world!")
    })
}

#[bench]
fn encode_str(b: &mut Bencher) {
    b.iter(|| {
        Encoder::encode("hello world!")
    })
}

#[bench]
fn decode_u64(b: &mut Bencher) {
    let buffer = Encoder::encode(::std::u64::MAX);

    b.iter(|| {
        let mut decoder = Decoder::new(&buffer);
        decoder.uint64().unwrap()
    })
}

#[bench]
fn decode_f64(b: &mut Bencher) {
    let buffer = Encoder::encode(3.141592653589793f64);

    b.iter(|| {
        let mut decoder = Decoder::new(&buffer);
        decoder.float64().unwrap()
    })
}
