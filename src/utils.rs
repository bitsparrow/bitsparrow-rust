use std::{error, fmt};
use std::str::Utf8Error;
use std::string::FromUtf8Error;

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

/// Simple error type returned either by the `Decoder` or `Encoder`
#[derive(Debug)]
pub enum Error {
    Utf8Encoding,
    ReadingOutOfBounds,
    BufferNotEmpty,
    InvalidData,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use Error::*;

        match *self {
            Utf8Encoding       => "Couldn't decode UTF-8 string",
            ReadingOutOfBounds => "Attempted to read out of bounds",
            BufferNotEmpty     => "There is still data to read",
            InvalidData        => "Data does not match requested type",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(error::Error::description(self))
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Error {
        Error::Utf8Encoding
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_: FromUtf8Error) -> Error {
        Error::Utf8Encoding
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
