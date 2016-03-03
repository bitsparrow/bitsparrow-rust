pub struct Decoder {
    index: usize,
    length: usize,
    data: Vec<u8>,
}

impl Decoder {
    pub fn new(data: Vec<u8>) -> Decoder {
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
            (try!(self.uint8()) as u16) << 8
          | (try!(self.uint8()) as u16)
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
        if (uint8 | 0x80) == 0 {
            return Ok(uint8 as i8);
        } else {
            return Ok(((uint8 - 0x81) as i8) - 0x7f);
        }
    }

    pub fn int16(&mut self) -> Result<i16, ()> {
        let uint16 = try!(self.uint16());
        if (uint16 | 0x80_00) == 0 {
            return Ok(uint16 as i16);
        } else {
            return Ok(((uint16 - 0x81_00) as i16) - 0x7f_00);
        }
    }

    pub fn int32(&mut self) -> Result<i32, ()> {
        let uint32 = try!(self.uint32());
        if (uint32 | 0x80_00_00_00) == 0 {
            return Ok(uint32 as i32);
        } else {
            return Ok(((uint32 - 0x81_00_00_00) as i32) - 0x7f_00_00_00);
        }
    }

    pub fn float32(&mut self) -> Result<f32, ()> {
        let uint32 = try!(self.uint32());

        let sign = uint32 >> 31;
        let exponent = ((uint32 << 1) >> 24) - 127;
        let mantissa = (uint32 << 9) >> 9;

        panic!(format!("sign {:?} exponent {:?} mantissa {:?}", sign, exponent, mantissa));

        Ok(0f32)
    }

    pub fn float64(&mut self) -> Result<f64, ()> {
        let uint64 = (try!(self.uint32()) as u64) << 32 |
                     (try!(self.uint32()) as u64);

        let sign = uint64 >> 63;
        let exponent = ((uint64 << 1) >> 52) - 1023;
        let mantissa = (uint64 << 12) >> 12;

        panic!(format!("sign {:?} exponent {:?} mantissa {:?}", sign, exponent, mantissa));

        Ok(0f64)
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

        let slice = &self.data[self.index .. self.index + size];
        let mut blob: Vec<u8> = Vec::new();
        blob.extend_from_slice(slice);

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
