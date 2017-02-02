#![feature(test)]
#![feature(proc_macro)]

extern crate test;
extern crate bitsparrow;
#[macro_use]
extern crate bitsparrow_derive;

use bitsparrow::*;

use test::Bencher;

#[derive(BitEncodable, BitDecodable, PartialEq, Debug)]
struct Foo {
    bar: String,
    baz: u64,
    derp: bool,
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
fn encode_derived_struct(b: &mut Bencher) {
    let foo = Foo {
        bar: "hello".into(),
        baz: 1337u64,
        derp: true,
    };

    b.iter(|| {
        Encoder::encode(&foo)
    })
}

#[bench]
fn encode_slice(b: &mut Bencher) {
    b.iter(|| {
        Encoder::encode(b"hello world!")
    })
}

#[bench]
fn encode_tuple(b: &mut Bencher) {
    b.iter(|| {
        Encoder::encode(("hello world!", 3.14f32, false))
    })
}

#[bench]
fn encode_complex_slice(b: &mut Bencher) {
    b.iter(|| {
        Encoder::encode(&[3.14f32, 2.15, 1.16])
    })
}

#[bench]
fn decode_complex_vec(b: &mut Bencher) {
    let buffer = Encoder::encode(&[3.14f32, 2.15, 1.16]);

    b.iter(|| {
        let _foo: Vec<f32> = Decoder::decode(&buffer).unwrap();
    })
}

#[bench]
fn decode_tuple(b: &mut Bencher) {
    let buffer = Encoder::encode((10u64, 3.14f32, true));

    b.iter(|| {
        let _foo: (u64, f32, bool) = Decoder::decode(&buffer).unwrap();
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
