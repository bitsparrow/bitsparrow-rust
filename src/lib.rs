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
//!              .end()
//!              .unwrap();
//!
//! assert_eq!(buffer, vec![0x64,0x03,0x46,0x6f,0x6f])
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
//! let buffer = encoder.string("Foo")
//!              .end()
//!              .unwrap();
//!
//! assert_eq!(buffer, vec![0x64,0x03,0x46,0x6f,0x6f]);
//! ```
//!
//! To make the monad chain feasible, Encoder will internally
//! store the last error (if any) that occures during the chain,
//! and return in on the `Result` of the `end` method.
//!
//! ## Decoding
//!
//! ```
//! use bitsparrow::Decoder;
//!
//! let buffer: Vec<u8> = vec![0x64,0x03,0x46,0x6f,0x6f];
//! let mut decoder = Decoder::new(buffer);
//!
//! assert_eq!(100u8, decoder.uint8().unwrap());
//! assert_eq!("Foo", decoder.string().unwrap());
//! assert_eq!(true, decoder.end());
//! ```
//!
//! Decoder consumes the buffer and allows you to retrieve the
//! values in order they were encoded. Calling the `end` method
//! is optional, it will return true if you have read the entire
//! buffer, which can be handy if you are reading multiple
//! messages stacked on a single buffer.

use std::{ mem, fmt, error, str };

/// Simple error type returned either by the `Decoder` or `Encoder`
#[derive(Debug)]
pub struct Error(String);

impl Error {
    pub fn new(msg: &str) -> Error {
        Error(msg.to_string())
    }

    pub fn out_of_bounds() -> Error {
        Error::new("Attempted to read out of bounds")
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        return &self.0;
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Encoder takes in typed data and produces a binary buffer
/// represented as `Vec<u8>`.
pub struct Encoder {
    data: Vec<u8>,
    bool_index: usize,
    bool_shift: u8,
    last_error: Option<Error>,
}

impl Encoder {
    /// Create a new instance of the `Encoder`.
    pub fn new() -> Encoder {
        Encoder {
            data: Vec::new(),
            bool_index: std::usize::MAX,
            bool_shift: 0,
            last_error: None,
        }
    }

    /// Store a `u8` on the buffer.
    pub fn uint8(&mut self, uint8: u8) -> &mut Encoder {
        self.data.push(uint8);
        return self;
    }

    /// Store a 'u16' on the buffer.
    pub fn uint16(&mut self, uint16: u16) -> &mut Encoder {
        self.data.reserve(2);
        self.data.push((uint16 >> 8) as u8);
        self.data.push((uint16 & 0xFF) as u8);
        return self;
    }

    /// Store a 'u32' on the buffer.
    pub fn uint32(&mut self, uint32: u32) -> &mut Encoder {
        self.data.reserve(4);
        self.data.push((uint32 >> 24) as u8);
        self.data.push(((uint32 >> 16) & 0xFF) as u8);
        self.data.push(((uint32 >> 8) & 0xFF) as u8);
        self.data.push((uint32 & 0xFF) as u8);
        return self;
    }

    /// Store an `i8` on the buffer.
    pub fn int8(&mut self, int8: i8) -> &mut Encoder {
        self.uint8(unsafe { mem::transmute_copy(&int8) })
    }

    /// Store an `i16` on the buffer.
    pub fn int16(&mut self, int16: i16) -> &mut Encoder {
        self.uint16(unsafe { mem::transmute_copy(&int16) })
    }

    /// Store an `i32` on the buffer.
    pub fn int32(&mut self, int32: i32) -> &mut Encoder {
        self.uint32(unsafe { mem::transmute_copy(&int32) })
    }

    /// Store a `float32` on the buffer.
    pub fn float32(&mut self, float32: f32) -> &mut Encoder {
        self.uint32(unsafe { mem::transmute_copy(&float32) })
    }

    /// Store a `float64` on the buffer.
    pub fn float64(&mut self, float64: f64) -> &mut Encoder {
        let uint64: u64 = unsafe { mem::transmute_copy(&float64) };
        return self
            .uint32((uint64 >> 32) as u32)
            .uint32((uint64 & 0xFFFFFFFF) as u32);
    }

    /// Store a `bool` on the buffer. Calling `bool` multiple times
    /// in a row will attempt to store the information on a single
    /// byte.
    ///
    /// ```
    /// use bitsparrow::Encoder;
    ///
    /// let buffer = Encoder::new()
    ///              .bool(true)
    ///              .bool(false)
    ///              .bool(false)
    ///              .bool(false)
    ///              .bool(false)
    ///              .bool(true)
    ///              .bool(true)
    ///              .bool(true)
    ///              .end()
    ///              .unwrap();
    ///
    /// // booleans are stacked as bits on a single byte, right to left.
    /// assert_eq!(vec![0b11100001], buffer);
    /// ```
    pub fn bool(&mut self, bool: bool) -> &mut Encoder {
        let bool_bit: u8 = if bool { 1 } else { 0 };
        let index = self.data.len();

        if self.bool_index == index && self.bool_shift < 7 {
            self.bool_shift += 1;
            self.data[index - 1] = self.data[index - 1] | bool_bit << self.bool_shift;
            return self;
        }

        self.bool_index = index + 1;
        self.bool_shift = 0;
        self.uint8(bool_bit)
    }

    /// Store a `usize` on the buffer. This will use a variable amount of bytes
    /// depending on the value of `usize`, making it a very powerful and flexible
    /// type to send around. BitSparrow uses `size` internally to prefix `string`
    /// and `bytes` as those can have an arbitrary length, and using a large
    /// number type such as u32 could be an overkill if all you want to send is
    /// `"Foo"`. Detailed explanation on how BitSparrow stores `size` can be found
    /// on [the homepage](http://bitsparrow.io).
    pub fn size(&mut self, size: usize) -> &mut Encoder {
        if size > 0x3FFFFFFF {
            self.last_error = Some(Error::new("[size] value is too large"));
            return self;
        }

        // can fit on 7 bits
        if size < 0x80 {
            return self.uint8(size as u8);
        }

        // can fit on 14 bits
        if size < 0x4000 {
            return self.uint16((size as u16) | 0x8000);
        }

        // use up to 30 bits
        return self.uint32((size as u32) | 0xC0000000);
    }

    /// Store an arbitary collection of bytes represented as `&[u8]`,
    /// easy to use by dereferencing `Vec<u8>` with `&`.
    pub fn bytes(&mut self, bytes: &[u8]) -> &mut Encoder {
        let size = bytes.len();
        if size > 0x3FFFFFFF {
            self.last_error = Some(Error::new("[bytes] is too long"));
            return self;
        }
        self.size(size);
        self.data.extend_from_slice(bytes);
        return self;
    }

    /// Store an arbitrary UTF-8 Rust string on the buffer.
    pub fn string(&mut self, string: &str) -> &mut Encoder {
        let size = string.len();
        if size > 0x3FFFFFFF {
            self.last_error = Some(Error::new("[string] is too long"));
            return self;
        }
        self.size(size);
        self.data.extend_from_slice(string.as_bytes());
        return self;
    }

    /// Finish encoding, resets the encoder
    pub fn end(&mut self) -> Result<Vec<u8>, Error> {
        let error = self.last_error.take();
        match error {
            Some(error) => Err(error),
            None        => {
                self.bool_index = std::usize::MAX;
                self.bool_shift = 0;

                Ok(mem::replace(&mut self.data, Vec::new()))
            }
        }
    }
}


/// Decoder consumes a buffer represented as `Vec<u8>` and exposes
/// methods to read BitSparrow types from it in the same order they
/// were encoded by the `Encoder`.
pub struct Decoder {
    index: usize,
    length: usize,
    data: Vec<u8>,
    bool_index: usize,
    bool_shift: u8,
}

impl Decoder {
    /// Consume a buffer represented as `Vec<u8>` and return a new
    /// instance of the `Decoder`.
    ///
    /// **Note:** Decoder does not mutate the buffer, but it needs to
    ///           progress it's internal state, hence it has to be mutable.
    ///           This also prevents it from being borrowed and used in
    ///           multiple places at the same time, which would be an
    ///           anti-pattern.
    pub fn new(data: Vec<u8>) -> Decoder {
        Decoder {
            index: 0,
            length: data.len(),
            data: data,
            bool_index: std::usize::MAX,
            bool_shift: 0,
        }
    }

    /// Read a `u8` from the buffer and progress the internal index.
    pub fn uint8(&mut self) -> Result<u8, Error> {
        if self.index >= self.length {
            return Err(Error::out_of_bounds());
        }
        let uint8 = self.data[self.index];
        self.index += 1;
        return Ok(uint8);
    }

    /// Read a `u16` from the buffer and progress the internal index.
    pub fn uint16(&mut self) -> Result<u16, Error> {
        Ok(
            (try!(self.uint8()) as u16) << 8 |
            (try!(self.uint8()) as u16)
        )
    }

    /// Read a `u32` from the buffer and progress the internal index.
    pub fn uint32(&mut self) -> Result<u32, Error> {
        Ok(
            (try!(self.uint8()) as u32) << 24 |
            (try!(self.uint8()) as u32) << 16 |
            (try!(self.uint8()) as u32) << 8  |
            (try!(self.uint8()) as u32)
        )
    }

    /// Read an `i8` from the buffer and progress the internal index.
    pub fn int8(&mut self) -> Result<i8, Error> {
        let uint8 = try!(self.uint8());
        Ok(unsafe { mem::transmute_copy(&uint8) })
    }

    /// Read an `i16` from the buffer and progress the internal index.
    pub fn int16(&mut self) -> Result<i16, Error> {
        let uint16 = try!(self.uint16());
        Ok(unsafe { mem::transmute_copy(&uint16) })
    }

    /// Read an `i32` from the buffer and progress the internal index.
    pub fn int32(&mut self) -> Result<i32, Error> {
        let uint32 = try!(self.uint32());
        Ok(unsafe { mem::transmute_copy(&uint32) })
    }

    /// Read a `float32` from the buffer and progress the internal index.
    pub fn float32(&mut self) -> Result<f32, Error> {
        let uint32 = try!(self.uint32());
        Ok(unsafe { mem::transmute_copy(&uint32) })
    }

    /// Read a `float64` from the buffer and progress the internal index.
    pub fn float64(&mut self) -> Result<f64, Error> {
        let uint64 = (try!(self.uint32()) as u64) << 32 |
                                  (try!(self.uint32()) as u64);
        Ok(unsafe { mem::transmute_copy(&uint64) })
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
    /// let mut decoder = Decoder::new(vec![0b11100001]);
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
    /// // Ensure we consumed the whole buffer
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
        return Ok(bits & 1 == 1);
    }

    /// Read a `usize` from the buffer and progress the index. Detailed
    /// explanation on how BitSparrow stores `size` can be found on
    /// [the homepage](http://bitsparrow.io).
    pub fn size(&mut self) -> Result<usize, Error> {
        let mut size: usize = try!(self.uint8()) as usize;

        // 1 byte (no signature)
        if (size & 128) == 0 {
            return Ok(size);
        }

        let sig: u8 = (size as u8) >> 6;
        // remove signature from the first byte
        size = size & 63 /* 00111111 */;

        // 2 bytes (signature is 10)
        if sig == 2 {
            return Ok(size << 8 | try!(self.uint8()) as usize);
        }

        Ok(
            size << 24                          |
            (try!(self.uint8()) as usize) << 16 |
            (try!(self.uint8()) as usize) << 8  |
            (try!(self.uint8()) as usize)
        )
    }

    /// Read an arbitary sized binary data from the buffer and
    /// progress the index.
    ///
    /// **Note:** BitSparrow internally prefixes `bytes` with
    /// `size` so you don't have to worry about how many bytes
    /// you need to read.
    pub fn bytes(&mut self) -> Result<&[u8], Error> {
        let size = try!(self.size());
        if self.index + size > self.length {
            return Err(Error::out_of_bounds());
        }

        let bytes = &self.data[self.index .. self.index + size];

        self.index += size;

        return Ok(bytes);
    }

    /// Read an arbitary sized owned `String` from the buffer and
    /// progress the index.
    ///
    /// **Note:** Analog to `bytes`, BitSparrow internally prefixes
    /// `string` with `size` so you don't have to worry about how
    /// many bytes you need to read.
    pub fn string(&mut self) -> Result<&str, Error> {
        let bytes = try!(self.bytes());
        return match str::from_utf8(bytes) {
            Ok(string) => Ok(string),
            Err(_) => Err(Error::new("Couldn't decode UTF-8 string")),
        }
    }

    /// Returns `true` if the entire buffer has been read, otherwise
    /// returns `false`.
    pub fn end(&self) -> bool {
        self.index >= self.length
    }
}
