
// tmp
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub use failure::Error;
pub type Result<T> = ::std::result::Result<T, Error>;

use ifd::{
    DataType,
    Entry,
};
use tag::TagKind;

#[derive(Debug, Fail)]
pub enum IncorrectDetail {
    #[fail(display = "No ByteOrder")]
    NoByteOrder,

    #[fail(display = "No ver42")]
    NoVersion,

    #[fail(display = "No IFD address")]
    NoIFDAddress,
}

#[derive(Debug, Fail)]
pub enum DecodeError {
    #[fail(display = "Incorrect: {:?}", detail)]
    IncorrectHeader{ detail: IncorrectDetail },

    #[fail(display = "Cannot find the tag ({:?}) in the IFD.", tag)]
    CannotFindTheTag { tag: TagKind },

    #[fail(display = "Unsupported IFD Entry: {:?}\n  because of: {:?}", entry, reason)]
    UnsupportedIFDEntry { entry: Entry, reason: String, },

    #[fail(display = "Unsupported data(u32): {:?}, in tag: {:?}", data, tag)]
    UnsupportedData { tag: TagKind, data: u32 },

    #[fail(display = "Want u8 data, but got {:?} [tag: {:?}]", data, tag)]
    UnwantedData { tag: TagKind, data: u32 },

    #[fail(display = "`SamplesPerPixel`({:?}) and the number of `BitsPerSample`({:?}) should be the same.", samples, bits)]
    NotMatchNumberOfSamples { samples: u8, bits: Vec<u8>, },

    #[fail(display = "No image")]
    NoImage,
}

pub enum EncodeError {}
