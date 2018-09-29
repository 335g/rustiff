// TODO: encoder impl
/// The `rustiff` crate provides TIFF format decoder and encoder.
/// 
/// # Brief overview
/// 
/// The primary types in this crate is [`Decoder`](struct.Decoder.html) for decoding TIFF data.
/// 
/// [`IFD`](struct.IFD.html) is used when multiple images are included in one file.

//#![warn(missing_docs)]

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
    PhotometricInterpretation,
};
