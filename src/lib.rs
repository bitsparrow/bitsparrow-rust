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
