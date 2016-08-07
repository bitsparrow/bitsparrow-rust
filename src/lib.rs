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

/// Encoder takes in typed data and produces a binary buffer
/// represented as `Vec<u8>`.
pub struct Encoder {
    data: Vec<u8>,
    bool_index: usize,
    bool_shift: u8,
}

macro_rules! write_bytes {
    ($data:expr, $value:ident) => ({
        unsafe {
            let size = mem::size_of_val(&$value);
            let ptr: *const u8 = mem::transmute(&$value.to_be());

            let len = $data.len();
            $data.reserve(size);
            $data.set_len(len + size);

            ptr::copy_nonoverlapping(
                ptr,
                $data.as_mut_ptr().offset(len as isize),
                size
            );
        }
    })
}

impl Encoder {
    /// Create a new instance of the `Encoder`.
    #[inline]
    pub fn new() -> Encoder {
        Encoder {
            data: Vec::new(),
            bool_index: std::usize::MAX,
            bool_shift: 0,
        }
    }

    /// Create a new instance of the `Encoder` with a preallocated buffer capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Encoder {
        Encoder {
            data: Vec::with_capacity(capacity),
            bool_index: std::usize::MAX,
            bool_shift: 0,
        }
    }

    /// Store a `u8` on the buffer.
    #[inline]
    pub fn uint8(&mut self, uint8: u8) -> &mut Encoder {
        self.data.push(uint8);

        self
    }

    /// Store a 'u16' on the buffer.
    #[inline]
    pub fn uint16(&mut self, uint16: u16) -> &mut Encoder {
        write_bytes!(self.data, uint16);

        self
    }

    /// Store a 'u32' on the buffer.
    #[inline]
    pub fn uint32(&mut self, uint32: u32) -> &mut Encoder {
        write_bytes!(self.data, uint32);

        self
    }

    /// Store a 'u64' on the buffer.
    #[inline]
    pub fn uint64(&mut self, uint64: u64) -> &mut Encoder {
        write_bytes!(self.data, uint64);

        self
    }

    /// Store an `i8` on the buffer.
    #[inline]
    pub fn int8(&mut self, int8: i8) -> &mut Encoder {
        self.data.push(int8 as u8);

        self
    }

    /// Store an `i16` on the buffer.
    #[inline]
    pub fn int16(&mut self, int16: i16) -> &mut Encoder {
        write_bytes!(self.data, int16);

        self
    }

    #[inline]
    /// Store an `i32` on the buffer.
    pub fn int32(&mut self, int32: i32) -> &mut Encoder {
        write_bytes!(self.data, int32);

        self
    }

    #[inline]
    /// Store an `i32` on the buffer.
    pub fn int64(&mut self, int64: i64) -> &mut Encoder {
        write_bytes!(self.data, int64);

        self
    }

    /// Store a `float32` on the buffer.
    #[inline]
    pub fn float32(&mut self, float32: f32) -> &mut Encoder {
        self.uint32(unsafe { mem::transmute(float32) })
    }

    /// Store a `float64` on the buffer.
    #[inline]
    pub fn float64(&mut self, float64: f64) -> &mut Encoder {
        self.uint64(unsafe { mem::transmute(float64) })
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
    ///              .end();
    ///
    /// // booleans are stacked as bits on a single byte, right to left.
    /// assert_eq!(buffer, &[0b11100001]);
    /// ```
    #[inline]
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
    #[inline]
    pub fn size(&mut self, size: usize) -> &mut Encoder {
        if size < 128 {
            return self.uint8(size as u8);
        }

        let mut size = size as u64;

        let lead = size.leading_zeros() as usize;
        let bytes = if lead == 0 { 9 } else { 9 - (lead - 1) / 7 };

        let mut buf: [u8; 9] = unsafe { mem::uninitialized() };

        for i in (1 .. bytes).rev() {
            buf[i] = size as u8;
            size >>= 8;
        }
        buf[0] = (size as u8) | SIZE_MASKS[bytes - 1];

        self.data.extend_from_slice(&buf[0 .. bytes]);

        self
    }

    /// Store an arbitary collection of bytes represented as `&[u8]`,
    /// easy to use by dereferencing `Vec<u8>` with `&`.
    #[inline]
    pub fn bytes(&mut self, bytes: &[u8]) -> &mut Encoder {
        self.size(bytes.len());
        self.data.extend_from_slice(bytes);

        self
    }

    /// Store an arbitrary UTF-8 Rust string on the buffer.
    #[inline]
    pub fn string(&mut self, string: &str) -> &mut Encoder {
        self.size(string.len());
        self.data.extend_from_slice(string.as_bytes());

        self
    }

    /// Finish encoding, obtain the buffer and reset the encoder.
    #[inline]
    pub fn end(&mut self) -> Vec<u8> {
        self.bool_index = std::usize::MAX;
        self.bool_shift = 0;

        mem::replace(&mut self.data, Vec::new())
    }
}


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
            let ptr: *mut u8 = mem::transmute(&mut value);

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
