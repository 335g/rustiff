#![allow(dead_code)]

mod data;
mod decode;
mod element;
mod encode;
mod error;
mod header;
mod ifd;
mod image;
mod possible;
pub mod tag;
mod val;

pub use decode::{Decoded, Decoder};
pub use error::{DecodeError, DecodeErrorKind, DecodeResult, DecodingError, FileHeaderError};
pub use val::{BitsPerSample, Compression, PhotometricInterpretation};
