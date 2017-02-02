//! For implementations in other languages, and more detailed
//! information on the types check out http://bitsparrow.io/.
//!
//! # BitSparrow in Rust
//!
//! ## Encoding
//!
//! ```
//! use bitsparrow::Encoder;
//!
//! let buffer = Encoder::new()
//!              .uint8(100)
//!              .string("Foo")
//!              .end();
//!
//! assert_eq!(buffer, &[0x64,0x03,0x46,0x6f,0x6f])
//! ```
//!
//! Each method on the `Encoder` will return a mutable borrow of
//! the encoder. If you need to break the monad chain, store the
//! owned encoder as a variable before writing to it, e.g.:
//!
//! ```
//! use bitsparrow::Encoder;
//!
//! let mut encoder = Encoder::new();
//! encoder.uint8(100);
//!
//! /*
//!  * Many codes here
//!  */
//!
//! let buffer = encoder.string("Foo").end();
//!
//! assert_eq!(buffer, &[0x64_u8,0x03,0x46,0x6f,0x6f]);
//! ```
//!
//! ## Decoding
//!
//! ```
//! use bitsparrow::Decoder;
//!
//! let buffer = &[0x64,0x03,0x46,0x6f,0x6f];
//! let mut decoder = Decoder::new(buffer);
//!
//! assert_eq!(100u8, decoder.uint8().unwrap());
//! assert_eq!("Foo", decoder.string().unwrap());
//! assert_eq!(true, decoder.end());
//! ```
//!
//! Decoder allows you to retrieve the values in order they were
//! encoded. Calling the `end` method is optional - it will return
//! `true` if you have read the entire buffer, ensuring the entire
//! buffer has been read.

mod encode;
mod decode;
mod utils;

pub use utils::Error;
pub use encode::{Encoder, BitEncodable};
pub use decode::{Decoder, BitDecodable};
