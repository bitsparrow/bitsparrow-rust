# BitSparrow in Rust

![](https://api.travis-ci.org/bitsparrow/bitsparrow-rust.svg)

**[Homepage](http://bitsparrow.io/) -**
**[API Documentation](http://bitsparrow.io/doc/bitsparrow/) -**
**[Cargo](https://crates.io/crates/bitsparrow)**

## Encoding

```rust
use bitsparrow::Encoder;

let buffer = Encoder::new()
             .uint8(100)
             .string("Foo")
             .end();

assert_eq!(buffer, &[0x64,0x03,0x46,0x6f,0x6f])
```

Each method on the `Encoder` will consume the instance of the
struct. If you need to break the monad chain, store the
intermediate state of the encoder, e.g.:

```rust
use bitsparrow::Encoder;

let encoder = Encoder::new();
encoder.uint8(100);

/* ... */

let buffer = encoder.string("Foo").end();

assert_eq!(buffer, &[0x64,0x03,0x46,0x6f,0x6f]);
```

## Decoding

```rust
use bitsparrow::Decoder;

let buffer = &[0x64,0x03,0x46,0x6f,0x6f];
let mut decoder = Decoder::new(buffer);

assert_eq!(100u8, decoder.uint8().unwrap());
assert_eq!("Foo", decoder.string().unwrap());
assert_eq!(true, decoder.end());
```

Decoder allows you to retrieve the values in order they were
encoded. Calling the `end` method is optional - it will return
`true` if you have read the entire buffer, ensuring the entire
buffer has been read.

## Performance

All primitive number types are encoded and decoded using straight
low level memory copying and type transmutations. Even on
little-endian hardware (the encoded data is always big-endian) the
cost of encoding/decoding is virtually none:

```
test allocate_8 ... bench:          26 ns/iter (+/- 4)
test decode_f64 ... bench:           0 ns/iter (+/- 0)
test decode_u64 ... bench:           0 ns/iter (+/- 0)
test encode_f64 ... bench:          26 ns/iter (+/- 6)
test encode_u64 ... bench:          26 ns/iter (+/- 3)
```

Encoding benchmark includes allocating 8 bytes on the heap, the
`allocate_8` just creates `Vec::with_capacity(8)` to demonstrate that
the actual encoding process is very, very cheap.

## The MIT License (MIT)

Copyright (c) 2016 BitSparrow

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
