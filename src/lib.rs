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

pub use encode::Encoder;

use std::{ mem, fmt, error, str, ptr };

/// Simple error type returned either by the `Decoder` or `Encoder`
#[derive(Debug)]
pub enum Error {
    Utf8Encoding,
    ReadingOutOfBounds,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Utf8Encoding       => "Couldn't decode UTF-8 string",
            Error::ReadingOutOfBounds => "Attempted to read out of bounds",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

static SIZE_MASKS: [u8; 9] = [
    0b00000000,
    0b10000000,
    0b11000000,
    0b11100000,
    0b11110000,
    0b11111000,
    0b11111100,
    0b11111110,
    0b11111111
];

/// Decoder reads from a binary slice buffer (`&[u8]`) and exposes
/// methods to read BitSparrow types from it in the same order they
/// were encoded by the `Encoder`.
pub struct Decoder<'a> {
    index: usize,
    data: &'a [u8],
    bool_index: usize,
    bool_shift: u8,
}

macro_rules! read_bytes {
    ($decoder:expr, $t:ident) => ({
        let size = mem::size_of::<$t>();
        let end = $decoder.index + size;
        if end > $decoder.data.len() {
            return Err(Error::ReadingOutOfBounds);
        }

        unsafe {
            let mut value: $t = mem::uninitialized();
            let ptr = &mut value as *mut $t as *mut u8;

            ptr::copy_nonoverlapping(
                $decoder.data.as_ptr().offset($decoder.index as isize),
                ptr,
                size
            );

            $decoder.index = end;

            Ok($t::from_be(value))
        }
    })
}

impl<'a> Decoder<'a> {
    /// Create a new `Decoder` reading from a `&[u8]` slice buffer.
    #[inline]
    pub fn new(data: &[u8]) -> Decoder {
        Decoder {
            index: 0,
            data: data,
            bool_index: std::usize::MAX,
            bool_shift: 0,
        }
    }

    /// Read a `u8` from the buffer and progress the internal index.
    #[inline]
    pub fn uint8(&mut self) -> Result<u8, Error> {
        if self.index >= self.data.len() {
            return Err(Error::ReadingOutOfBounds);
        }
        let uint8 = self.data[self.index];
        self.index += 1;
        return Ok(uint8);
    }

    /// Read a `u16` from the buffer and progress the internal index.
    #[inline]
    pub fn uint16(&mut self) -> Result<u16, Error> {
        read_bytes!(self, u16)
    }

    /// Read a `u32` from the buffer and progress the internal index.
    #[inline]
    pub fn uint32(&mut self) -> Result<u32, Error> {
        read_bytes!(self, u32)
    }

    /// Read a `u64` from the buffer and progress the internal index.
    #[inline]
    pub fn uint64(&mut self) -> Result<u64, Error> {
        read_bytes!(self, u64)
    }

    /// Read an `i8` from the buffer and progress the internal index.
    #[inline]
    pub fn int8(&mut self) -> Result<i8, Error> {
        let uint8 = try!(self.uint8());

        Ok(uint8 as i8)
    }

    /// Read an `i16` from the buffer and progress the internal index.
    #[inline]
    pub fn int16(&mut self) -> Result<i16, Error> {
        read_bytes!(self, i16)
    }

    /// Read an `i32` from the buffer and progress the internal index.
    #[inline]
    pub fn int32(&mut self) -> Result<i32, Error> {
        read_bytes!(self, i32)
    }

    /// Read an `i64` from the buffer and progress the internal index.
    #[inline]
    pub fn int64(&mut self) -> Result<i64, Error> {
        read_bytes!(self, i64)
    }

    /// Read a `float32` from the buffer and progress the internal index.
    #[inline]
    pub fn float32(&mut self) -> Result<f32, Error> {
        let uint32 = try!(self.uint32());

        Ok(unsafe { mem::transmute(uint32) })
    }

    /// Read a `float64` from the buffer and progress the internal index.
    #[inline]
    pub fn float64(&mut self) -> Result<f64, Error> {
        let uint64 = try!(self.uint64());

        Ok(unsafe { mem::transmute(uint64) })
    }

    /// Read a `bool` from the buffer and progress the internal index. If
    /// a `bool` was previously read from the buffer, calling `bool()`
    /// on the `Decoder` again will read a boolean from the same index
    /// without progressing, but instead shifting to read the next bit.
    /// This behavior is symmetric to how the `Encoder` stores the `bool`s,
    /// and is completely transparent when using the API.
    ///
    /// ```
    /// use bitsparrow::Decoder;
    ///
    /// // Reading `bools` from a single byte.
    /// let buffer = &[0b11100001];
    /// let mut decoder = Decoder::new(buffer);
    ///
    /// assert_eq!(true, decoder.bool().unwrap());
    /// assert_eq!(false, decoder.bool().unwrap());
    /// assert_eq!(false, decoder.bool().unwrap());
    /// assert_eq!(false, decoder.bool().unwrap());
    /// assert_eq!(false, decoder.bool().unwrap());
    /// assert_eq!(true, decoder.bool().unwrap());
    /// assert_eq!(true, decoder.bool().unwrap());
    /// assert_eq!(true, decoder.bool().unwrap());
    ///
    /// // Ensure we've read the entire buffer
    /// assert_eq!(true, decoder.end());
    /// ```
    pub fn bool(&mut self) -> Result<bool, Error> {
        if self.bool_index == self.index && self.bool_shift < 7 {
            self.bool_shift += 1;
            let bits = self.data[self.index - 1];
            let bool_bit = 1 << self.bool_shift;
            return Ok(bits & bool_bit == bool_bit);
        }

        let bits = try!(self.uint8());
        self.bool_index = self.index;
        self.bool_shift = 0;

        Ok(bits & 1 == 1)
    }

    /// Read a `usize` from the buffer and progress the index. Detailed
    /// explanation on how BitSparrow stores `size` can be found on
    /// [the homepage](http://bitsparrow.io).
    pub fn size(&mut self) -> Result<usize, Error> {
        let high = try!(self.uint8());

        // 1 byte (no signature)
        if (high & 128) == 0 {
            return Ok(high as usize);
        }

        let mut ext_bytes = (!high).leading_zeros() as usize;
        let mut size = (high ^ SIZE_MASKS[ext_bytes]) as usize;

        while ext_bytes != 0 {
            ext_bytes -= 1;
            size = (size << 8) | try!(self.uint8()) as usize;
        }

        Ok(size)
    }

    /// Read an arbitary sized binary data from the buffer and
    /// progress the index.
    ///
    /// **Note:** BitSparrow internally prefixes `bytes` with
    /// `size` so you don't have to worry about how many bytes
    /// you need to read.
    #[inline]
    pub fn bytes(&mut self) -> Result<&[u8], Error> {
        // Order of addition is important here!
        // Calling `size` will modify the `index`.
        let end = try!(self.size()) + self.index;

        if end > self.data.len() {
            return Err(Error::ReadingOutOfBounds);
        }

        let bytes = &self.data[self.index .. end];

        self.index = end;

        Ok(bytes)
    }

    /// Read an arbitary sized owned `String` from the buffer and
    /// progress the index.
    ///
    /// **Note:** Analog to `bytes`, BitSparrow internally prefixes
    /// `string` with `size` so you don't have to worry about how
    /// many bytes you need to read.
    #[inline]
    pub fn string(&mut self) -> Result<&str, Error> {
        str::from_utf8(try!(self.bytes())).map_err(|_| Error::Utf8Encoding)
    }

    /// Returns `true` if the entire buffer has been read, otherwise
    /// returns `false`.
    #[inline]
    pub fn end(&self) -> bool {
        self.index >= self.data.len()
    }
}
