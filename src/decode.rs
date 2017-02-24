use std::{mem, ptr};
use std::str::from_utf8;

use utils::{SIZE_MASKS, Error, Result};

/// Decoder reads from a binary slice buffer (`&[u8]`) and exposes
/// methods to read BitSparrow types from it in the same order they
/// were encoded by the `Encoder`.
pub struct Decoder<'src> {
    index: usize,
    data: &'src [u8],
    ptr: *const u8,
    bool_index: usize,
    bool_shift: u8,
}

pub trait BitDecode<'src>: Sized + 'src {
    fn decode(&mut Decoder<'src>) -> Result<Self>;
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


impl<'src> Decoder<'src> {
    /// Create a new `Decoder` reading from a `&[u8]` slice buffer.
    #[inline]
    pub fn new(data: &[u8]) -> Decoder {
        Decoder {
            index: 0,
            data: data,
            ptr: data.as_ptr(),
            bool_index: ::std::usize::MAX,
            bool_shift: 0,
        }
    }

    #[inline]
    pub fn decode<D: BitDecode<'src>>(data: &'src [u8]) -> Result<D> {
        let mut d = Decoder::new(data);
        let value = try!(BitDecode::decode(&mut d));
        if !d.end() {
            return Err(Error::BufferNotEmpty);
        }
        Ok(value)
    }

    #[inline]
    pub fn read<D: BitDecode<'src>>(&mut self) -> Result<D> {
        BitDecode::decode(self)
    }

    /// Read a `u8` from the buffer and progress the internal index.
    #[inline]
    pub fn uint8(&mut self) -> Result<u8> {
        if self.index >= self.data.len() {
            return Err(Error::ReadingOutOfBounds);
        }
        let uint8 = unsafe { *self.ptr.offset(self.index as isize) };
        self.index += 1;
        return Ok(uint8);
    }

    /// Read a `u16` from the buffer and progress the internal index.
    #[inline]
    pub fn uint16(&mut self) -> Result<u16> {
        read_bytes!(self, u16)
    }

    /// Read a `u32` from the buffer and progress the internal index.
    #[inline]
    pub fn uint32(&mut self) -> Result<u32> {
        read_bytes!(self, u32)
    }

    /// Read a `u64` from the buffer and progress the internal index.
    #[inline]
    pub fn uint64(&mut self) -> Result<u64> {
        read_bytes!(self, u64)
    }

    /// Read an `i8` from the buffer and progress the internal index.
    #[inline]
    pub fn int8(&mut self) -> Result<i8> {
        let uint8 = try!(self.uint8());

        Ok(uint8 as i8)
    }

    /// Read an `i16` from the buffer and progress the internal index.
    #[inline]
    pub fn int16(&mut self) -> Result<i16> {
        read_bytes!(self, i16)
    }

    /// Read an `i32` from the buffer and progress the internal index.
    #[inline]
    pub fn int32(&mut self) -> Result<i32> {
        read_bytes!(self, i32)
    }

    /// Read an `i64` from the buffer and progress the internal index.
    #[inline]
    pub fn int64(&mut self) -> Result<i64> {
        read_bytes!(self, i64)
    }

    /// Read a `float32` from the buffer and progress the internal index.
    #[inline]
    pub fn float32(&mut self) -> Result<f32> {
        let uint32 = try!(self.uint32());

        Ok(unsafe { mem::transmute(uint32) })
    }

    /// Read a `float64` from the buffer and progress the internal index.
    #[inline]
    pub fn float64(&mut self) -> Result<f64> {
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
    #[inline]
    pub fn bool(&mut self) -> Result<bool> {
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
    #[inline(always)]
    pub fn size(&mut self) -> Result<usize> {
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
    pub fn bytes(&mut self) -> Result<&'src [u8]> {
        // Order of addition is important here!
        // Calling `size` will modify the `index`.
        let len = try!(self.size());
        let end = len + self.index;

        if end > self.data.len() {
            return Err(Error::ReadingOutOfBounds);
        }

        let bytes = unsafe { ::std::slice::from_raw_parts(
            self.ptr.offset(self.index as isize),
            len
        ) };

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
    pub fn string(&mut self) -> Result<&'src str> {
        from_utf8(try!(self.bytes())).map_err(Into::into)
    }

    /// Returns `true` if the entire buffer has been read, otherwise
    /// returns `false`.
    #[inline]
    pub fn end(&self) -> bool {
        self.index >= self.data.len()
    }
}

macro_rules! impl_decodable {
    ($func:ident, $t:ty) => {
        impl<'src> BitDecode<'src> for $t {
            #[inline]
            fn decode(d: &mut Decoder<'src>) -> Result<Self> {
                d.$func()
            }
        }
    }
}

impl_decodable!(uint16, u16);
impl_decodable!(uint32, u32);
impl_decodable!(uint64, u64);
impl_decodable!(int8, i8);
impl_decodable!(int16, i16);
impl_decodable!(int32, i32);
impl_decodable!(int64, i64);
impl_decodable!(float32, f32);
impl_decodable!(float64, f64);
impl_decodable!(bool, bool);
impl_decodable!(size, usize);

impl<'src> BitDecode<'src> for &'src [u8] {
    #[inline]
    fn decode(d: &mut Decoder<'src>) -> Result<Self> {
        d.bytes()
    }
}

impl<'src> BitDecode<'src> for Vec<u8> {
    #[inline]
    fn decode(d: &mut Decoder<'src>) -> Result<Self> {
        // Order of addition is important here!
        // Calling `size` will modify the `index`.
        let len = try!(d.size());
        let end = len + d.index;

        if end > d.data.len() {
            return Err(Error::ReadingOutOfBounds);
        }

        let mut vec = Vec::with_capacity(len);

        unsafe {
            ::std::ptr::copy_nonoverlapping(
                d.ptr.offset(d.index as isize),
                vec.as_mut_ptr(),
                len
            );

            vec.set_len(len);
        }

        d.index = end;

        Ok(vec)
    }
}

impl<'src> BitDecode<'src> for &'src str {
    #[inline]
    fn decode(d: &mut Decoder<'src>) -> Result<&'src str> {
        d.string()
    }
}

impl<'src> BitDecode<'src> for String {
    #[inline]
    fn decode(d: &mut Decoder<'src>) -> Result<Self> {
        String::from_utf8(try!(BitDecode::decode(d))).map_err(Into::into)
    }
}

impl<'src, D: BitDecode<'src>> BitDecode<'src> for Vec<D> {
    #[inline]
    fn decode(d: &mut Decoder<'src>) -> Result<Self> {
        let size = try!(d.size());

        let mut vec = Vec::with_capacity(size);

        for _ in 0..size {
            vec.push(try!(D::decode(d)));
        }

        Ok(vec)
    }
}

macro_rules! impl_tuple {
    ($( $l:ident ),*) => {
        impl<'src, $($l),*> BitDecode<'src> for ($($l),*) where
            $(
                $l: BitDecode<'src>,
            )*
        {
            #[inline(always)]
            fn decode(d: &mut Decoder<'src>) -> Result<Self> {
                Ok(( $( try!($l::decode(d)) ),* ))
            }
        }
    }
}

impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);
impl_tuple!(A, B, C, D, E);
impl_tuple!(A, B, C, D, E, F);
impl_tuple!(A, B, C, D, E, F, G);
impl_tuple!(A, B, C, D, E, F, G, H);
impl_tuple!(A, B, C, D, E, F, G, H, I);
impl_tuple!(A, B, C, D, E, F, G, H, I, J);
