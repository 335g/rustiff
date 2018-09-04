
extern crate byteorder;
extern crate lzw;
#[macro_use] extern crate failure;

mod error;
mod byte;
mod decode;
mod ifd;
mod image;
pub mod tag;

pub use decode::Decoder;
pub use ifd::IFD;
pub use error::{
    DecodeError,
    DecodeErrorKind,
    DecodeResult,
};
pub use image::{
    Image,
    ImageData,
    ImageHeader,
    ImageHeaderError,
    Compression,
    BitsPerSample,
    BitsPerSampleError,
    PhotometricInterpretation,
};
