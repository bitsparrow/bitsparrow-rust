# BitSparrow in Rust

[Homepage](http://bitsparrow.io/) - [API Documentation](http://bitsparrow.io/doc/bitsparrow/)

For implementations in other languages, and more detailed
information on the types check out http://bitsparrow.io/.

# BitSparrow in Rust


Encoder takes typed data in order and produces a binary
buffer represented as a `Vec<u8>`.

## Encoding

```rust
use bitsparrow::Encoder;

let buffer = Encoder::new()
.uint8(100)
.string("Foo")
.end()
.unwrap();

assert_eq!(buffer, vec![0x64,0x03,0x46,0x6f,0x6f])
```

Each method on the `Encoder` will consume the instance of the
struct. If you need to break the monad chain, store the
intermediate state of the encoder, e.g.:

```rust
use bitsparrow::Encoder;

let encoder = Encoder::new()
.uint8(100);

/*
* Many codes here
*/

let buffer = encoder.string("Foo")
.end()
.unwrap();

assert_eq!(buffer, vec![0x64,0x03,0x46,0x6f,0x6f]);
```

To make the monad chain feasible, Encoder will internally
store the last error (if any) that occures during the chain,
and return in on the `Result` of the `end` method.

## Decoding

```rust
use bitsparrow::Decoder;

let buffer: Vec<u8> = vec![0x64,0x03,0x46,0x6f,0x6f];
let mut decoder = Decoder::new(buffer);

assert_eq!(100u8, decoder.uint8().unwrap());
assert_eq!("Foo", decoder.string().unwrap());
assert_eq!(true, decoder.end());
```

Decoder consumes the buffer and allows you to retrieve the
values in order they were encoded. Calling the `end` method
is optional, it will return true if you have read the entire
buffer, which can be handy if you are reading multiple
messages stacked on a single buffer.

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
