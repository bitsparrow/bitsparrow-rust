use std::mem;
use std::fmt;
use std::error::Error;

///
/// # ReadError
///
/// Returned by the Decoder on failed attempts to read
/// outside of the buffer size.
///
pub struct ReadError;

impl Error for ReadError {
    fn description(&self) -> &str {
        return "Failed to read ouf of the buffer";
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Debug for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

///
/// #EncodingError
///
/// Returned by the Encoder when a value fails to encode.
///
pub struct EncodingError<'a>(&'a str);

impl<'a> Error for EncodingError<'a> {
    fn description(&self) -> &str {
        return self.0;
    }
}

impl<'a> fmt::Display for EncodingError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl<'a> fmt::Debug for EncodingError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

pub struct Encoder<'a> {
    chunks: Vec<Chunk<'a>>,
    capacity: usize,
}

enum Chunk<'a> {
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Blob(&'a [u8]),
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

    pub fn size(&'a mut self, size: usize) -> &'a mut Encoder {
        if size > 0x3FFFFFFF {
            return self.uint32(0xFFFFFFFF);
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
        let borrow = self.size(size);
        borrow.capacity += size;
        borrow.chunks.push(Chunk::Blob(blob));
        return borrow;
    }

    pub fn string(&'a mut self, string: &'a str) -> &'a mut Encoder {
        self.blob(string.as_bytes())
    }

    pub fn encode(&'a self) -> Result<Vec<u8>, EncodingError> {
        let mut data: Vec<u8> = Vec::with_capacity(self.capacity);

        for chunk in self.chunks.iter() {
            match chunk {
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
                &Chunk::Blob(blob) => {
                    if blob.len() > 0x3FFFFFFF {
                        return Err(EncodingError("Trying to encode too long data"));
                    }
                    data.extend_from_slice(blob);
                }
            }
        }

        Ok(data)
    }
}

pub struct Decoder<'a> {
    index: usize,
    length: usize,
    data: &'a[u8],
}

impl<'a> Decoder<'a> {
    pub fn new(data: &[u8]) -> Decoder {
        Decoder {
            index: 0,
            length: data.len(),
            data: data,
        }
    }

    pub fn uint8(&mut self) -> Result<u8, ReadError> {
        if self.index >= self.length {
            return Err(ReadError);
        }
        let uint8 = self.data[self.index];
        self.index += 1;
        return Ok(uint8);
    }

    pub fn uint16(&mut self) -> Result<u16, ReadError> {
        Ok(
            (try!(self.uint8()) as u16) << 8 |
            (try!(self.uint8()) as u16)
        )
    }

    pub fn uint32(&mut self) -> Result<u32, ReadError> {
        Ok(
            (try!(self.uint8()) as u32) << 24 |
            (try!(self.uint8()) as u32) << 16 |
            (try!(self.uint8()) as u32) << 8  |
            (try!(self.uint8()) as u32)
        )
    }

    pub fn int8(&mut self) -> Result<i8, ReadError> {
        let uint8 = try!(self.uint8());
        Ok(unsafe { mem::transmute_copy(&uint8) })
    }

    pub fn int16(&mut self) -> Result<i16, ReadError> {
        let uint16 = try!(self.uint16());
        Ok(unsafe { mem::transmute_copy(&uint16) })
    }

    pub fn int32(&mut self) -> Result<i32, ReadError> {
        let uint32 = try!(self.uint32());
        Ok(unsafe { mem::transmute_copy(&uint32) })
    }

    pub fn float32(&mut self) -> Result<f32, ReadError> {
        let uint32 = try!(self.uint32());
        Ok(unsafe { mem::transmute_copy(&uint32) })
    }

    pub fn float64(&mut self) -> Result<f64, ReadError> {
        let uint64 = (try!(self.uint32()) as u64) << 32 |
                     (try!(self.uint32()) as u64);
        Ok(unsafe { mem::transmute_copy(&uint64) })
    }

    pub fn size(&mut self) -> Result<usize, ReadError> {
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

    pub fn blob(&mut self) -> Result<Vec<u8>, ReadError> {
        let size = try!(self.size());
        if self.index + size >= self.length {
            return Err(ReadError);
        }

        let blob = self.data[self.index .. self.index + size].to_vec();

        self.index += size;

        return Ok(blob);
    }

    pub fn string(&mut self) -> Result<String, ReadError> {
        let blob = try!(self.blob());
        return match String::from_utf8(blob) {
            Ok(string) => Ok(string),
            Err(_) => Err(ReadError),
        }
    }
}
