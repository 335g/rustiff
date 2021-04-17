#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod byte;
mod data;
mod decode;
mod dir;
// mod encode;
mod error;
mod num;

#[macro_use]
mod macros;

pub mod tag;
pub mod val;

pub use data::{Data, DataType, Entry};
pub use decode::{Decoded, Decoder};
pub use error::{
    DecodeError, DecodeErrorKind, DecodeResult, DecodingError, FileHeaderError, TagError,
};
pub use val::{
    BitsPerSample, Compression, PhotometricInterpretation, Rational,
    Value, Values,
};
pub use num::{
    Tone, DynamicTone,
};
