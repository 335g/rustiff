
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
    CannotFindTheTag { tag: Tag },

    #[fail(display = "Unsupported data type for the tag, (tag: {:?}, datatype: {:?}", datatype, tag)]
    UnsupportedDataTypeForThisTag { tag: Tag, datatype: DataType },

    #[fail(display = "Unsupported data type: {:?}", datatype)]
    UnsupportedDataType { datatype: DataType },
}

pub enum EncodeError {}
