use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::{convert::From, num::TryFromIntError};

use crate::dir::DataType;
use crate::tag::{AnyTag, Tag};

pub type DecodeResult<T> = std::result::Result<T, DecodeError>;

#[derive(Debug, thiserror::Error)]
pub struct DecodeError(#[from] DecodeErrorDetail);

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ERROR to decode tiff format\n\
            - {:?}",
            self.0
        )
    }
}

impl DecodeError {
    pub(crate) fn new(detail: DecodeErrorDetail) -> Self {
        Self(detail)
    }

    pub fn kind(&self) -> &DecodeErrorDetail {
        &self.0
    }
}

impl From<io::Error> for DecodeError {
    fn from(err: io::Error) -> DecodeError {
        DecodeError(DecodeErrorDetail::Io(err))
    }
}

impl From<FileHeaderError> for DecodeError {
    fn from(detail: FileHeaderError) -> DecodeError {
        let detail = DecodeErrorDetail::FileHeader(detail);
        DecodeError::from(detail)
    }
}

impl From<DecodeValueError> for DecodeError {
    fn from(detail: DecodeValueError) -> Self {
        let detail = DecodeErrorDetail::Value(detail);
        DecodeError::from(detail)
    }
}

impl From<std::num::TryFromIntError> for DecodeError {
    fn from(err: std::num::TryFromIntError) -> DecodeError {
        let detail = DecodeValueError::Overflow(err);
        DecodeError::from(detail)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeErrorDetail {
    #[error("IO\n-- {0:?}")]
    Io(#[from] io::Error),

    #[error("header\n-- {0:?}")]
    FileHeader(#[from] FileHeaderError),

    #[error("value\n-- {0:?}")]
    Value(#[from] DecodeValueError),

    #[error("tag\n-- {0:?}")]
    Tag(#[from] TagError),
}

#[derive(Debug, thiserror::Error)]
pub enum FileHeaderError {
    /// Tiff file header has 2 byte data at the beginning.
    /// This error occurs when there is no 2 byte data.
    #[error(
        "No byte order\n\
        --- Tiff file header has 2 byte data at the beginning. \
        This error occurs when there is no 2 byte data"
    )]
    NoByteOrder,

    /// Tiff file header has 2 byte data at the beginning.
    /// 2 byte data should be b'II' or b'MM'.
    /// This error occurs when 2 byte data is incorrect data.
    #[error(
        "Invalid byte order: {byte_order:?}\n\
        --- Tiff file header has 2 byte data at the beginning. \
        2 byte data should be b'II' or b'MM'. \
        This error occurs when 2 byte data is incorrect data."
    )]
    InvalidByteOrder {
        #[allow(missing_docs)]
        byte_order: [u8; 2],
    },

    /// There is `0x00 0x2A` data after data corresponding to byte order.
    /// This error occurs when there is no this 2 byte data.
    #[error(
        "No vision\n\
        --- There is `0x00 0x2A` data after data corresponding to byte order.\
        This error occurs when there is no this 2 byte data."
    )]
    NoVersion,

    /// There is `0x00 0x2A` data after data corresponding to byte order.
    /// This error occurs when 2 byte data is not equal 42.
    #[error(
        "Invalid version: {version:?}\n\
        --- There is `0x00 0x2A` data after data corresponding to byte order.\
        This error occurs when 2 byte data is not equal 42."
    )]
    InvalidVersion {
        #[allow(missing_docs)]
        version: u16,
    },

    /// There is 4 byte data corresponding to an address of Image File Directory (IFD).
    /// This error occurs when there is no this 4 byte data.
    #[error(
        "No IFD address\n\
        --- There is 4 byte data corresponding to an address of Image File Directory (IFD).\
        This error occurs when there is no this 4 byte data."
    )]
    NoIFDAddress,
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeValueError {
    #[error("Invalid value: {0:?}")]
    InvalidValue(Vec<u32>),

    #[error("Invalid count: {0:?}")]
    InvalidCount(u32),

    #[error("Invalid data type: {0:?}")]
    InvalidDataType(DataType),

    #[error("Overflow\n---{0:?}")]
    Overflow(#[from] std::num::TryFromIntError),

    #[error("No value that should be")]
    NoValueThatShouldBe,
}

#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("Cannot find the tag")]
    CannotFindTag,
}
