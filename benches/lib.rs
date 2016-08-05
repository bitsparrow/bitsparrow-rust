#![feature(test)]

extern crate test;
extern crate bitsparrow;
use bitsparrow::{ Encoder, Decoder };

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
        Encoder::new().uint64(::std::u64::MAX).end().unwrap()
    })
}


#[bench]
fn encode_f64(b: &mut Bencher) {
    b.iter(|| {
        Encoder::new().float64(3.141592653589793).end().unwrap()
    })
}


#[bench]
fn decode_u64(b: &mut Bencher) {
    let buffer = Encoder::new().uint64(::std::u64::MAX).end().unwrap();

    b.iter(|| {
        let mut decoder = Decoder::new(&buffer);
        decoder.uint64().unwrap()
    })
}
