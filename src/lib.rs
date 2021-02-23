#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod byte;
mod data;
mod decode;
mod dir;
// mod encode;
mod error;

#[macro_use]
mod macros;

pub mod tag;
pub mod val;

pub use decode::{Decoded, Decoder};
pub use dir::DataType;
pub use error::{
    DecodeError, DecodeErrorKind, DecodeResult, DecodeValueError, FileHeaderError, TagError,
};
pub use val::{
    BitsPerSample, Byte, Bytes, Compression, Long, Longs, PhotometricInterpretation, Rational,
    Short, Shorts, Value, Values,
};
