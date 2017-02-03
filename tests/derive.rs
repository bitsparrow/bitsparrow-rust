extern crate bitsparrow;
#[macro_use]
extern crate bitsparrow_derive;

use bitsparrow::*;

#[derive(BitEncode, BitDecode, PartialEq, Debug)]
struct Foo {
    bar: Vec<Bar>,
    baz: String,
    derp: bool,
}

#[derive(BitEncode, BitDecode, PartialEq, Debug)]
struct Bar {
    value: u64
}

#[test]
fn encode_foo() {
    let foo = Foo {
        bar: vec![Bar { value: 10 }, Bar { value: 1337 }],
        baz: "Hello world".into(),
        derp: true,
    };

    let buffer = Encoder::encode(&foo);

    let decoded: (Vec<u64>, String, bool) = Decoder::decode(&buffer).unwrap();

    assert_eq!(decoded, (vec![10, 1337], "Hello world".into(), true));
}

#[test]
fn decode_foo() {
    let buffer = Encoder::encode(([10u64, 1337], "Hello world", true));

    let foo: Foo = Decoder::decode(&buffer).unwrap();

    assert_eq!(foo, Foo {
        bar: vec![Bar { value: 10 }, Bar { value: 1337 }],
        baz: "Hello world".into(),
        derp: true,
    });
}
