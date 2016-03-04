use std::mem;

pub struct Encoder {
    data: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Encoder {
        Encoder {
            data: Vec::new(),
        }
    }

    pub fn uint8(&mut self, uint8: u8) -> &mut Encoder {
        self.data.push(uint8);
        return self;
    }

    pub fn uint16(&mut self, uint16: u16) -> &mut Encoder {
        self.data.reserve(2);
        self.data.push((uint16 >> 8) as u8);
        self.data.push((uint16 & 0xFF) as u8);
        return self;
    }

    pub fn uint32(&mut self, uint32: u32) -> &mut Encoder {
        self.data.reserve(4);
        self.data.push((uint32 >> 24) as u8);
        self.data.push(((uint32 >> 16) & 0xFF) as u8);
        self.data.push(((uint32 >> 8) & 0xFF) as u8);
        self.data.push((uint32 & 0xFF) as u8);
        return self;
    }

    pub fn int8(&mut self, int8: i8) -> &mut Encoder {
        self.uint8(unsafe { mem::transmute_copy(&int8) })
    }

    pub fn int16(&mut self, int16: i16) -> &mut Encoder {
        self.uint16(unsafe { mem::transmute_copy(&int16) })
    }

    pub fn int32(&mut self, int32: i32) -> &mut Encoder {
        self.uint32(unsafe { mem::transmute_copy(&int32) })
    }

    pub fn float32(&mut self, float32: f32) -> &mut Encoder {
        self.uint32(unsafe { mem::transmute_copy(&float32) })
    }

    pub fn float64(&mut self, float64: f64) -> &mut Encoder {
        let uint64: u64 = unsafe { mem::transmute_copy(&float64) };
        return self
            .uint32((uint64 >> 32) as u32)
            .uint32((uint64 & 0xFFFFFFFF) as u32);
    }

    pub fn size(&mut self, size: usize) -> &mut Encoder {
        if size > 0x3fffffff {
            return self.uint32(0xffffffff);
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
        return self.uint32((size as u32) | 0xc0000000);
    }

    pub fn blob(&mut self, blob: &[u8]) -> &mut Encoder {
        let mut size = blob.len();
        if size > 0x3fffffff {
            size = 0x3fffffff;
        }
        self.size(size);
        self.data.extend_from_slice(&blob[..size]);
        return self;
    }

    pub fn string(&mut self, string: &str) -> &mut Encoder {
        self.blob(string.as_bytes())
    }

    pub fn encode(&self) -> Vec<u8> {
        return self.data.clone();
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

    pub fn uint8(&mut self) -> Result<u8, ()> {
        if self.index >= self.length {
            return Err(());
        }
        let uint8 = self.data[self.index];
        self.index += 1;
        return Ok(uint8);
    }

    pub fn uint16(&mut self) -> Result<u16, ()> {
        Ok(
            (try!(self.uint8()) as u16) << 8 |
            (try!(self.uint8()) as u16)
        )
    }

    pub fn uint32(&mut self) -> Result<u32, ()> {
        Ok(
            (try!(self.uint8()) as u32) << 24 |
            (try!(self.uint8()) as u32) << 16 |
            (try!(self.uint8()) as u32) << 8  |
            (try!(self.uint8()) as u32)
        )
    }

    pub fn int8(&mut self) -> Result<i8, ()> {
        let uint8 = try!(self.uint8());
        Ok(unsafe { mem::transmute_copy(&uint8) })
    }

    pub fn int16(&mut self) -> Result<i16, ()> {
        let uint16 = try!(self.uint16());
        Ok(unsafe { mem::transmute_copy(&uint16) })
    }

    pub fn int32(&mut self) -> Result<i32, ()> {
        let uint32 = try!(self.uint32());
        Ok(unsafe { mem::transmute_copy(&uint32) })
    }

    pub fn float32(&mut self) -> Result<f32, ()> {
        let uint32 = try!(self.uint32());
        Ok(unsafe { mem::transmute_copy(&uint32) })
    }

    pub fn float64(&mut self) -> Result<f64, ()> {
        let uint64 = (try!(self.uint32()) as u64) << 32 |
                     (try!(self.uint32()) as u64);
        Ok(unsafe { mem::transmute_copy(&uint64) })
    }

    pub fn size(&mut self) -> Result<usize, ()> {
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

    pub fn blob(&mut self) -> Result<Vec<u8>, ()> {
        let size = try!(self.size());
        if self.index + size >= self.length {
            return Err(());
        }

        let blob = self.data[self.index .. self.index + size].to_vec();

        self.index += size;

        return Ok(blob);
    }

    pub fn string(&mut self) -> Result<String, ()> {
        let blob = try!(self.blob());
        return match String::from_utf8(blob) {
            Ok(string) => Ok(string),
            Err(_) => Err(()),
        }
    }
}
