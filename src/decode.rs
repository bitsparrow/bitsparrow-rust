
/// Decoder reads from a binary slice buffer (`&[u8]`) and exposes
/// methods to read BitSparrow types from it in the same order they
/// were encoded by the `Encoder`.
pub struct Decoder<'a> {
    index: usize,
    data: &'a [u8],
}
