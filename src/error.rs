
// tmp
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub use failure::Error;
pub type Result<T> = ::std::result::Result<T, Error>;

use ifd::{
    Tag,
    DataType,
};

#[derive(Debug, Fail)]
pub enum DecodeError {
    #[fail(display = "Incorrect header : ({:?})", reason)]
    IncorrectHeader { reason: String, },

    #[fail(display = "Cannot decode byte code to IFD.")]
    NoIFD,

    #[fail(display = "Cannot decode byte code to IFD entry.")]
    NoIFDEntry,

    #[fail(display = "Cannot find the tag ({:?}) in the IFD.", tag)]
    CannotFindTheTag { tag: Tag },

    #[fail(display = "The value for this tag ({:?}) is many in the data field.", tag)]
    ALot { tag: Tag, },

    #[fail(display = "There are only a few values for this tag ({:?}) in the data field.", tag)]
    Few { tag: Tag, },

    #[fail(display = "Unsupported data type ({:?}) for ifd::Entry", datatype)]
    UnsupportedDataType { datatype: DataType },
}

pub enum EncodeError {}
