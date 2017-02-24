use std::{ptr, mem};
use utils::SIZE_MASKS;

/// Encoder takes in typed data and produces a binary buffer
/// represented as `Vec<u8>`.
pub struct Encoder {
    data: Vec<u8>,
    bool_index: usize,
    bool_shift: u8,
}

pub trait BitEncode {
    fn encode(&self, &mut Encoder);

    #[inline(always)]
    fn size_hint() -> usize {
        0
    }
}

impl Encoder {
    /// Create a new instance of the `Encoder`.
    #[inline(always)]
    pub fn new() -> Encoder {
        Encoder {
            data: Vec::new(),
            bool_index: ::std::usize::MAX,
            bool_shift: 0,
        }
    }

    /// Create a new instance of the `Encoder` with a preallocated buffer capacity.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Encoder {
        Encoder {
            data: Vec::with_capacity(capacity),
            bool_index: ::std::usize::MAX,
            bool_shift: 0,
        }
    }

    #[inline]
    pub fn encode<E: BitEncode>(val: E) -> Vec<u8> {
        let mut e = Encoder::with_capacity(E::size_hint());
        val.encode(&mut e);
        e.data
    }

    /// Store any type implementing `BitEncode` on the buffer.
    #[inline]
    pub fn write<E: BitEncode>(&mut self, val: E) -> &mut Self {
        val.encode(self);

        self
    }

    /// Store a `u8` on the buffer.
    #[inline]
    pub fn uint8(&mut self, val: u8) -> &mut Self {
        self.data.push(val);

        self
    }

    /// Store a 'u16' on the buffer.
    #[inline]
    pub fn uint16(&mut self, val: u16) -> &mut Self {
        val.encode(self);

        self
    }

    /// Store a 'u32' on the buffer.
    #[inline]
    pub fn uint32(&mut self, val: u32) -> &mut Self {
        val.encode(self);

        self
    }

    /// Store a 'u64' on the buffer.
    #[inline]
    pub fn uint64(&mut self, val: u64) -> &mut Self {
        val.encode(self);

        self
    }

    /// Store an `i8` on the buffer.
    #[inline]
    pub fn int8(&mut self, val: i8) -> &mut Self {
        val.encode(self);

        self
    }

    /// Store an `i16` on the buffer.
    #[inline]
    pub fn int16(&mut self, val: i16) -> &mut Self {
        val.encode(self);

        self
    }

    #[inline]
    /// Store an `i32` on the buffer.
    pub fn int32(&mut self, val: i32) -> &mut Self {
        val.encode(self);

        self
    }

    #[inline]
    /// Store an `i32` on the buffer.
    pub fn int64(&mut self, val: i64) -> &mut Self {
        val.encode(self);

        self
    }

    /// Store an `f32` on the buffer.
    #[inline]
    pub fn float32(&mut self, val: f32) -> &mut Self {
        val.encode(self);

        self
    }

    /// Store an `f64` on the buffer.
    #[inline]
    pub fn float64(&mut self, val: f64) -> &mut Self {
        val.encode(self);

        self
    }

    /// Store a `bool` on the buffer. Calling `bool` multiple times
    /// in a row will attempt to store the information on a single
    /// byte.
    ///
    /// ```
    /// use bitsparrow::Encoder;
    ///
    /// let buffer = Encoder::new()
    ///                     .bool(true)
    ///                     .bool(false)
    ///                     .bool(false)
    ///                     .bool(false)
    ///                     .bool(false)
    ///                     .bool(true)
    ///                     .bool(true)
    ///                     .bool(true)
    ///                     .end();
    ///
    /// // booleans are stacked as bits on a single byte, right to left.
    /// assert_eq!(buffer, &[0b11100001]);
    /// ```
    #[inline]
    pub fn bool(&mut self, val: bool) -> &mut Self {
        val.encode(self);

        self
    }

    /// Store a `usize` on the buffer. This will use a variable amount of bytes
    /// depending on the value of `usize`, making it a very powerful and flexible
    /// type to send around. BitSparrow uses `size` internally to prefix `string`
    /// and `bytes` as those can have an arbitrary length, and using a large
    /// number type such as u32 could be an overkill if all you want to send is
    /// `"Foo"`. Detailed explanation on how BitSparrow stores `size` can be found
    /// on [the homepage](http://bitsparrow.io).
    #[inline]
    pub fn size(&mut self, val: usize) -> &mut Self {
        self.size_with_reserve(val, 0);

        self
    }

    /// Store an arbitary collection of bytes represented as `&[u8]`,
    /// easy to use by dereferencing `Vec<u8>` with `&`.
    #[inline]
    pub fn bytes(&mut self, val: &[u8]) -> &mut Self {
        val.encode(self);

        self
    }

    /// Store an arbitrary UTF-8 Rust string on the buffer.
    #[inline]
    pub fn string(&mut self, val: &str) -> &mut Self {
        val.encode(self);

        self
    }

    /// Finish encoding, obtain the buffer and reset the encoder.
    #[inline(always)]
    pub fn end(&mut self) -> Vec<u8> {
        self.bool_index = ::std::usize::MAX;
        self.bool_shift = 0;

        mem::replace(&mut self.data, Vec::new())
    }

    #[inline(always)]
    fn size_with_reserve(&mut self, size: usize, item_size: usize) {
        if size < 128 {
            // Encoding size means data will follow, so it makes sense to reserve
            // capacity on the buffer beforehand
            self.data.reserve(1 + size * item_size);
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
        self.data.reserve(bytes + size * item_size);
        self.data.extend_from_slice(&buf[0 .. bytes]);
    }
}

// impl BitEncode for u8 {
//     #[inline(always)]
//     fn encode(&self, e: &mut Encoder) {
//         e.data.push(*self);
//     }
// }

impl BitEncode for i8 {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        e.data.push(*self as u8);
    }
}

macro_rules! impl_encodable {
    ($t:ty) => {
        impl BitEncode for $t {
            #[inline(always)]
            fn encode(&self, e: &mut Encoder) {
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

            #[inline(always)]
            fn size_hint() -> usize {
                mem::size_of::<$t>()
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

impl BitEncode for f32 {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        BitEncode::encode(&unsafe { mem::transmute::<f32, u32>(*self) }, e);
    }

    #[inline(always)]
    fn size_hint() -> usize {
        8
    }
}

impl BitEncode for f64 {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        BitEncode::encode(&unsafe { mem::transmute::<f64, u64>(*self) }, e);
    }

    #[inline(always)]
    fn size_hint() -> usize {
        8
    }
}

impl BitEncode for bool {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        let bit = *self as u8;
        let index = e.data.len();

        if e.bool_index == index && e.bool_shift < 7 {
            e.bool_shift += 1;
            e.data[index - 1] |= bit << e.bool_shift;
        } else {
            e.bool_index = index + 1;
            e.bool_shift = 0;
            e.data.push(bit);
        }
    }
}

impl BitEncode for usize {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        e.size_with_reserve(*self, 0);
    }
}

impl BitEncode for [u8] {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        e.size_with_reserve(self.len(), 1);

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

    #[inline(always)]
    fn size_hint() -> usize {
        16
    }
}

macro_rules! impl_deref {
    ($t:ty, $size:expr) => {
        impl<'a> BitEncode for &'a $t {
            #[inline(always)]
            fn encode(&self, e: &mut Encoder) {
                BitEncode::encode(*self, e);
            }

            #[inline(always)]
            fn size_hint() -> usize {
                $size
            }
        }
    }
}

impl_deref!(u16, 2);
impl_deref!(u32, 4);
impl_deref!(u64, 8);
impl_deref!(i8, 1);
impl_deref!(i16, 2);
impl_deref!(i32, 4);
impl_deref!(i64, 8);
impl_deref!(f32, 4);
impl_deref!(f64, 8);
impl_deref!(usize, 1);
impl_deref!(bool, 1);
impl_deref!([u8], 16);

macro_rules! impl_array {
    ($( $size:expr ),*) => {
        $(
            impl BitEncode for [u8; $size] {
                #[inline(always)]
                fn encode(&self, e: &mut Encoder) {
                    BitEncode::encode(AsRef::<[u8]>::as_ref(self), e);
                }

                #[inline(always)]
                fn size_hint() -> usize {
                    $size + 1
                }
            }

            impl<'a> BitEncode for &'a [u8; $size] {
                #[inline(always)]
                fn encode(&self, e: &mut Encoder) {
                    BitEncode::encode(AsRef::<[u8]>::as_ref(self), e);
                }

                #[inline(always)]
                fn size_hint() -> usize {
                    $size + 1
                }
            }

            impl<E: BitEncode> BitEncode for [E; $size] {
                #[inline(always)]
                fn encode(&self, e: &mut Encoder) {
                    BitEncode::encode(AsRef::<[E]>::as_ref(self), e);
                }

                #[inline(always)]
                fn size_hint() -> usize {
                    $size * E::size_hint() + 1
                }
            }

            impl<'a, E: BitEncode> BitEncode for &'a [E; $size] {
                #[inline(always)]
                fn encode(&self, e: &mut Encoder) {
                    BitEncode::encode(AsRef::<[E]>::as_ref(self), e);
                }

                #[inline(always)]
                fn size_hint() -> usize {
                    $size * E::size_hint() + 1
                }
            }
        )*
    }
}

impl_array!(
     0,
     1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
);

impl<'a> BitEncode for &'a Vec<u8> {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        BitEncode::encode(AsRef::<[u8]>::as_ref(*self), e);
    }

    #[inline(always)]
    fn size_hint() -> usize {
        16
    }
}

impl<'a> BitEncode for &'a str {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        BitEncode::encode(self.as_bytes(), e);
    }

    #[inline(always)]
    fn size_hint() -> usize {
        16
    }
}

impl BitEncode for String {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        BitEncode::encode(self.as_bytes(), e);
    }

    #[inline(always)]
    fn size_hint() -> usize {
        16
    }
}

impl<'a> BitEncode for &'a String {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        BitEncode::encode(self.as_bytes(), e);
    }

    #[inline(always)]
    fn size_hint() -> usize {
        16
    }
}

impl<E: BitEncode> BitEncode for [E] {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        e.size_with_reserve(self.len(), E::size_hint());
        for item in self {
            BitEncode::encode(item, e);
        }
    }
}

impl<'a, E: BitEncode> BitEncode for &'a [E] {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        BitEncode::encode(*self, e);
    }
}

impl<E: BitEncode> BitEncode for Vec<E> {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        BitEncode::encode(AsRef::<[E]>::as_ref(self), e);
    }
}

impl<'a, E: BitEncode> BitEncode for &'a Vec<E> {
    #[inline(always)]
    fn encode(&self, e: &mut Encoder) {
        BitEncode::encode(AsRef::<[E]>::as_ref(*self), e);
    }
}

macro_rules! impl_tuple {
    ($( $l:ident: $n:tt ),*) => {
        impl<$($l),*> BitEncode for ($($l),*) where
            $(
                $l: BitEncode,
            )*
        {
            #[inline(always)]
            fn encode(&self, e: &mut Encoder) {
                e.data.reserve(Self::size_hint());

                $(
                    self.$n.encode(e);
                )*
            }

            #[inline]
            fn size_hint() -> usize {
                $( $l::size_hint() + )* 0
            }
        }

        impl<'a, $($l),*> BitEncode for &'a ($($l),*) where
            $(
                $l: BitEncode,
            )*
        {
            #[inline(always)]
            fn encode(&self, e: &mut Encoder) {
                BitEncode::encode(*self, e);
            }

            #[inline]
            fn size_hint() -> usize {
                $( $l::size_hint() + )* 0
            }
        }
    }
}


impl_tuple!(A: 0, B: 1);
impl_tuple!(A: 0, B: 1, C: 2);
impl_tuple!(A: 0, B: 1, C: 2, D: 3);
impl_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4);
impl_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
impl_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
impl_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);
impl_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8);
impl_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9);
