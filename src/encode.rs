use std::{ptr, mem};

pub static SIZE_MASKS: [u8; 9] = [
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
    data: Vec<u8>
}

pub trait BitEncodable {
    #[inline]
    fn encode(self, &mut Encoder);
}

impl Encoder {
    /// Create a new instance of the `Encoder`.
    #[inline]
    pub fn new() -> Encoder {
        Encoder {
            data: Vec::new()
        }
    }

    /// Create a new instance of the `Encoder` with a preallocated buffer capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Encoder {
        Encoder {
            data: Vec::with_capacity(capacity)
        }
    }

    #[inline]
    pub fn encode<E: BitEncodable>(item: E) -> Vec<u8> {
        let mut e = Encoder::new();
        item.encode(&mut e);
        e.end()
    }

    #[inline]
    pub fn push<E: BitEncodable>(&mut self, item: E) -> &mut Self {
        item.encode(self);
        self
    }

    #[inline]
    pub fn end(self) -> Vec<u8> {
        self.data
    }

    #[inline]
    fn size(&mut self, size: usize, item: usize) {
        if size < 128 {
            // Encoding size means data will follow, so it makes sense to reserve
            // capacity on the buffer beforehand
            self.data.reserve(1 + size * item);
            self.data.push(size as u8);
            return;
        }

        let mut masked = size as u64;

        let lead = masked.leading_zeros() as usize;
        let bytes = if lead == 0 { 9 } else { 9 - (lead - 1) / 7 };

        let mut buf: [u8; 9] = unsafe { mem::uninitialized() };

        for i in (1 .. bytes).rev() {
            buf[i] = masked as u8;
            masked >>= 8;
        }
        buf[0] = (masked as u8) | SIZE_MASKS[bytes - 1];

        // Same as above...
        self.data.reserve(bytes + size * item);
        self.data.extend_from_slice(&buf[0 .. bytes]);
    }
}

// impl BitEncodable for u8 {
//     #[inline]
//     fn encode(self, e: &mut Encoder) {
//         e.data.push(self);
//     }
// }

impl BitEncodable for i8 {
    #[inline]
    fn encode(self, e: &mut Encoder) {
        e.data.push(self as u8);
    }
}

macro_rules! impl_encodable {
    ($t:ty) => {
        impl BitEncodable for $t {
            #[inline]
            fn encode(self, e: &mut Encoder) {
                unsafe {
                    let ptr: *const u8 = mem::transmute(&self.to_be());

                    let len = e.data.len();
                    e.data.reserve(mem::size_of::<$t>());
                    e.data.set_len(len + mem::size_of::<$t>());

                    ptr::copy_nonoverlapping(
                        ptr,
                        e.data.as_mut_ptr().offset(len as isize),
                        mem::size_of::<$t>()
                    );
                }
            }
        }
    }
}

impl_encodable!(u16);
impl_encodable!(u32);
impl_encodable!(u64);
impl_encodable!(i16);
impl_encodable!(i32);
impl_encodable!(i64);

impl BitEncodable for f32 {
    #[inline]
    fn encode(self, e: &mut Encoder) {
        BitEncodable::encode(unsafe { mem::transmute::<f32, u32>(self) }, e);
    }
}

impl BitEncodable for f64 {
    #[inline]
    fn encode(self, e: &mut Encoder) {
        BitEncodable::encode(unsafe { mem::transmute::<f64, u64>(self) }, e);
    }
}

impl BitEncodable for usize {
    /// Store a `usize` on the buffer. This will use a variable amount of bytes
    /// depending on the value of `usize`, making it a very powerful and flexible
    /// type to send around. BitSparrow uses `size` internally to prefix `string`
    /// and `bytes` as those can have an arbitrary length, and using a large
    /// number type such as u32 could be an overkill if all you want to send is
    /// `"Foo"`. Detailed explanation on how BitSparrow stores `size` can be found
    /// on [the homepage](http://bitsparrow.io).
    #[inline]
    fn encode(self, e: &mut Encoder) {
        e.size(self, 0);
    }
}

impl BitEncodable for bool {
    #[inline]
    fn encode(self, e: &mut Encoder) {
        e.data.push(self as u8);
    }
}

impl<'a> BitEncodable for &'a [u8] {
    #[inline(always)]
    fn encode(self, e: &mut Encoder) {
        e.size(self.len(), 1);

        unsafe {
            let len = e.data.len();

            // Encoder::size must reserve capacity beforehand
            debug_assert!(e.data.capacity() >= len + self.len());

            ptr::copy_nonoverlapping(
                self.as_ptr(),
                e.data.as_mut_ptr().offset(len as isize),
                self.len()
            );

            e.data.set_len(len + self.len());
        }
    }
}

// impl<'a, B: BitEncodable> BitEncodable for &'a B {
//     #[inline(always)]
//     fn encode(self, e: &mut Encoder) {
//         self.encode(e);
//     }
// }

// impl<'a, B: BitEncodable> BitEncodable for &'a [B] {
//     #[inline(always)]
//     fn encode(self, e: &mut Encoder) {
//         e.size(self.len(), 1);
//         for item in self {
//             BitEncodable::encode(item, e);
//         }
//     }
// }

impl<'a, T: AsRef<[u8]>> BitEncodable for &'a T {
    #[inline(always)]
    fn encode(self, e: &mut Encoder) {
        BitEncodable::encode(self.as_ref(), e);
    }
}

impl<'a> BitEncodable for &'a str {
    #[inline(always)]
    fn encode(self, e: &mut Encoder) {
        BitEncodable::encode(self.as_bytes(), e);
    }
}
