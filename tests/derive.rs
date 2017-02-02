#![feature(custom_derive)]
#![feature(proc_macro)]

extern crate bitsparrow;
#[macro_use] extern crate bitsparrow_derive;

use bitsparrow::*;

#[derive(BitEncodable, BitDecodable, PartialEq, Debug)]
struct Foo {
    bar: String,
    baz: u64,
    derp: bool,
}

#[test]
fn encode_foo() {
    let buffer = Encoder::encode(Foo {
        bar: "hello".into(),
        baz: 1337u64,
        derp: true,
    });

    let mut decoder = Decoder::new(&buffer);

    assert_eq!(decoder.string().unwrap(), "hello");
    assert_eq!(decoder.uint64().unwrap(), 1337);
    assert_eq!(decoder.bool().unwrap(), true);
    assert_eq!(decoder.end(), true);
}

#[test]
fn decode_foo() {
    let buffer = Encoder::encode(("hello", 1337u64, true));

    let foo: Foo = Decoder::decode(&buffer).unwrap();

    assert_eq!(foo, Foo {
        bar: "hello".into(),
        baz: 1337,
        derp: true,
    });
}
