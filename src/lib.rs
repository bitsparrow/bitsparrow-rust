use std::mem;
use std::fmt;
use std::error;

///
/// #EncodingError
///
/// Returned by the Encoder when a value fails to encode.
///
#[derive(Debug)]
pub struct Error<'a>(&'a str);

impl<'a> Error<'a> {
    pub fn out_of_bounds() -> Self {
        Error("Attempted to read out of bounds")
    }
}

impl<'a> error::Error for Error<'a> {
    fn description(&self) -> &str {
        return self.0;
    }
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Encoder<'a> {
    chunks: Vec<Chunk<'a>>,
    capacity: usize,
}

enum Chunk<'a> {
    Bool(u8, u8),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Blob(&'a [u8]),
    Error(&'a str),
}

impl<'a> Encoder<'a> {
    pub fn new() -> Encoder<'a> {
        Encoder {
            chunks: Vec::new(),
            capacity: 0,
        }
    }

    pub fn uint8(&'a mut self, uint8: u8) -> &'a mut Encoder {
        self.chunks.push(Chunk::Uint8(uint8));
        self.capacity += 1;
        return self;
    }

    pub fn uint16(&'a mut self, uint16: u16) -> &'a mut Encoder {
        self.chunks.push(Chunk::Uint16(uint16));
        self.capacity += 2;
        return self;
    }

    pub fn uint32(&'a mut self, uint32: u32) -> &'a mut Encoder {
        self.chunks.push(Chunk::Uint32(uint32));
        self.capacity += 4;
        return self;
    }

    pub fn int8(&'a mut self, int8: i8) -> &'a mut Encoder {
        self.uint8(unsafe { mem::transmute_copy(&int8) })
    }

    pub fn int16(&'a mut self, int16: i16) -> &'a mut Encoder {
        self.uint16(unsafe { mem::transmute_copy(&int16) })
    }

    pub fn int32(&'a mut self, int32: i32) -> &'a mut Encoder {
        self.uint32(unsafe { mem::transmute_copy(&int32) })
    }

    pub fn float32(&'a mut self, float32: f32) -> &'a mut Encoder {
        self.uint32(unsafe { mem::transmute_copy(&float32) })
    }

    pub fn float64(&'a mut self, float64: f64) -> &'a mut Encoder {
        let uint64: u64 = unsafe { mem::transmute_copy(&float64) };
        return self
            .uint32((uint64 >> 32) as u32)
            .uint32((uint64 & 0xFFFFFFFF) as u32);
    }

    pub fn bool(&'a mut self, bool: bool) -> &'a mut Encoder {
        let bool_bit: u8 = if bool { 1 } else { 0 };
        if self.chunks.is_empty() {
            self.chunks.push(Chunk::Bool(bool_bit, 0));
            return self;
        }

        let last_chunk = self.chunks.pop().unwrap();
        match last_chunk {
            Chunk::Bool(bits, shift) => {
                if shift < 7 {
                    let nshift = shift + 1;
                    self.chunks.push(Chunk::Bool(bits | (bool_bit << nshift), nshift));
                    return self;
                }
                // restore last chunk
                self.chunks.push(last_chunk);
            },
            // restore last chunk
            _ => self.chunks.push(last_chunk),
        }

        self.chunks.push(Chunk::Bool(bool_bit, 0));
        return self;
    }

    pub fn size(&'a mut self, size: usize) -> &'a mut Encoder {
        if size > 0x3FFFFFFF {
            self.chunks.push(Chunk::Error("[size] value is too large"));
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

    pub fn blob(&'a mut self, blob: &'a [u8]) -> &'a mut Encoder {
        let size = blob.len();
        if blob.len() > 0x3FFFFFFF {
            self.chunks.push(Chunk::Error("[blob] value is too long"));
            return self;
        }
        let sref = self.size(size);
        sref.capacity += size;
        sref.chunks.push(Chunk::Blob(blob));
        return sref;
    }

    pub fn string(&'a mut self, string: &'a str) -> &'a mut Encoder {
        self.blob(string.as_bytes())
    }

    pub fn encode(&'a self) -> Result<Vec<u8>, Error> {
        let mut data: Vec<u8> = Vec::with_capacity(self.capacity);

        for chunk in self.chunks.iter() {
            match chunk {
                &Chunk::Bool(bits, _) => data.push(bits),
                &Chunk::Uint8(uint8) => data.push(uint8),
                &Chunk::Uint16(uint16) => {
                    data.push((uint16 >> 8) as u8);
                    data.push((uint16 & 0xFF) as u8);
                },
                &Chunk::Uint32(uint32) => {
                    data.push(((uint32) >> 24) as u8);
                    data.push((((uint32) >> 16) & 0xFF) as u8);
                    data.push((((uint32) >> 8) & 0xFF) as u8);
                    data.push(((uint32) & 0xFF) as u8);
                },
                &Chunk::Blob(blob) => data.extend_from_slice(blob),
                &Chunk::Error(msg) => return Err(Error(msg)),
            }
        }

        Ok(data)
    }
}

pub struct Decoder<'a> {
    index: usize,
    length: usize,
    data: &'a[u8],
    bool_index: usize,
    bool_shift: u8,
}

impl<'a> Decoder<'a> {
    pub fn new(data: &[u8]) -> Decoder {
        Decoder {
            index: 0,
            length: data.len(),
            data: data,
            bool_index: std::usize::MAX,
            bool_shift: 0,
        }
    }

    pub fn uint8(&mut self) -> Result<u8, Error<'a>> {
        if self.index >= self.length {
            return Err(Error::out_of_bounds());
        }
        let uint8 = self.data[self.index];
        self.index += 1;
        return Ok(uint8);
    }

    pub fn uint16(&mut self) -> Result<u16, Error<'a>> {
        Ok(
            (try!(self.uint8()) as u16) << 8 |
            (try!(self.uint8()) as u16)
        )
    }

    pub fn uint32(&mut self) -> Result<u32, Error<'a>> {
        Ok(
            (try!(self.uint8()) as u32) << 24 |
            (try!(self.uint8()) as u32) << 16 |
            (try!(self.uint8()) as u32) << 8  |
            (try!(self.uint8()) as u32)
        )
    }

    pub fn int8(&mut self) -> Result<i8, Error<'a>> {
        let uint8 = try!(self.uint8());
        Ok(unsafe { mem::transmute_copy(&uint8) })
    }

    pub fn int16(&mut self) -> Result<i16, Error<'a>> {
        let uint16 = try!(self.uint16());
        Ok(unsafe { mem::transmute_copy(&uint16) })
    }

    pub fn int32(&mut self) -> Result<i32, Error<'a>> {
        let uint32 = try!(self.uint32());
        Ok(unsafe { mem::transmute_copy(&uint32) })
    }

    pub fn float32(&mut self) -> Result<f32, Error<'a>> {
        let uint32 = try!(self.uint32());
        Ok(unsafe { mem::transmute_copy(&uint32) })
    }

    pub fn float64(&mut self) -> Result<f64, Error<'a>> {
        let uint64 = (try!(self.uint32()) as u64) << 32 |
                     (try!(self.uint32()) as u64);
        Ok(unsafe { mem::transmute_copy(&uint64) })
    }

    pub fn bool(&mut self) -> Result<bool, Error<'a>> {
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

    pub fn size(&mut self) -> Result<usize, Error<'a>> {
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

    pub fn blob(&mut self) -> Result<Vec<u8>, Error<'a>> {
        let size = try!(self.size());
        if self.index + size >= self.length {
            return Err(Error::out_of_bounds());
        }

        let blob = self.data[self.index .. self.index + size].to_vec();

        self.index += size;

        return Ok(blob);
    }

    pub fn string(&mut self) -> Result<String, Error<'a>> {
        let blob = try!(self.blob());
        return match String::from_utf8(blob) {
            Ok(string) => Ok(string),
            Err(_) => Err(Error("Couldn't decode UTF-8 string")),
        }
    }
}
