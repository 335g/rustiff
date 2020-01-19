use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io;

use crate::ifd::DataType;
use crate::tag::Tag;

pub type DecodeResult<T> = std::result::Result<T, DecodeError>;

#[derive(Debug)]
pub struct DecodeError(DecodeErrorKind);

impl DecodeError {
    pub(crate) fn new(kind: DecodeErrorKind) -> DecodeError {
        DecodeError(kind)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Decode error: {:?}", self.0)
    }
}

impl Error for DecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<io::Error> for DecodeError {
    fn from(err: io::Error) -> DecodeError {
        DecodeError(DecodeErrorKind::Io(err))
    }
}

impl From<FileHeaderErrorDetail> for DecodeError {
    fn from(detail: FileHeaderErrorDetail) -> DecodeError {
        let kind = DecodeErrorKind::FileHeader(detail);
        DecodeError::new(kind)
    }
}

impl From<DecodeValueErrorDetail> for DecodeError {
    fn from(detail: DecodeValueErrorDetail) -> DecodeError {
        let kind = DecodeErrorKind::Value(detail);
        DecodeError::new(kind)
    }
}

impl From<TagErrorKind> for DecodeError {
    fn from(kind: TagErrorKind) -> DecodeError {
        let kind = DecodeErrorKind::Tag(kind);
        DecodeError::new(kind)
    }
}

impl From<std::num::TryFromIntError> for DecodeError {
    fn from(err: std::num::TryFromIntError) -> DecodeError {
        let kind = DecodeValueErrorDetail::Overflow(err);
        DecodeError::from(kind)
    }
}

#[derive(Debug)]
pub enum DecodeErrorKind {
    Io(io::Error),
    FileHeader(FileHeaderErrorDetail),
    Value(DecodeValueErrorDetail),
    Tag(TagErrorKind),
}

impl fmt::Display for DecodeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeErrorKind::Io(ref io) => write!(f, "IO error: {:?}", io),
            DecodeErrorKind::FileHeader(ref detail) => write!(f, "File header error: {:?}", detail),
            DecodeErrorKind::Value(ref detail) => write!(f, "Decode value error: {:?}", detail),
            DecodeErrorKind::Tag(ref detail) => write!(f, "Tag error: {:?}", detail),
        }
    }
}

impl Error for DecodeErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DecodeErrorKind::Io(ref io) => Some(io),
            DecodeErrorKind::FileHeader(ref detail) => Some(detail),
            DecodeErrorKind::Value(ref detail) => Some(detail),
            DecodeErrorKind::Tag(ref detail) => Some(detail),
        }
    }
}

#[derive(Debug)]
pub enum FileHeaderErrorDetail {
    /// Tiff file header has 2 byte data at the beginning.
    /// This error occurs when there is no 2 byte data.
    NoByteOrder,

    /// Tiff file header has 2 byte data at the beginning.
    /// 2 byte data should be b'II' or b'MM'.
    /// This error occurs when 2 byte data is incorrect data.
    InvalidByteOrder {
        #[allow(missing_docs)]
        byte_order: [u8; 2],
    },

    /// There is `0x00 0x2A` data after data corresponding to byte order.
    /// This error occurs when there is no this 2 byte data.
    NoVersion,

    /// There is `0x00 0x2A` data after data corresponding to byte order.
    /// This error occurs when 2 byte data is not equal 42.
    InvalidVersion {
        #[allow(missing_docs)]
        version: u16,
    },

    /// There is 4 byte data corresponding to an address of Image File Directory (IFD).
    /// This error occurs when there is no this 4 byte data.
    NoIFDAddress,
}

impl fmt::Display for FileHeaderErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FileHeaderErrorDetail::NoByteOrder => write!(f, "Incorrect header: No Byte Order"),
            FileHeaderErrorDetail::InvalidByteOrder { byte_order } => write!(
                f,
                "byte_order(`{:?}`) must be `b`II`` or `b`MM``",
                byte_order
            ),
            FileHeaderErrorDetail::NoVersion => write!(f, "No version, version must be 42."),
            FileHeaderErrorDetail::InvalidVersion { version } => {
                write!(f, "Version must be 42, but version is {:?}", version)
            }
            FileHeaderErrorDetail::NoIFDAddress => write!(f, "No IFD address"),
        }
    }
}

impl Error for FileHeaderErrorDetail {}

#[derive(Debug)]
pub enum DecodeValueErrorDetail {
    InvalidValue(Vec<u32>),
    InvalidCount(u32),
    InvalidDataType(DataType),
    Overflow(std::num::TryFromIntError),
}

impl fmt::Display for DecodeValueErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeValueErrorDetail::InvalidValue(ref values) => {
                write!(f, "Invalid value: {:?}", values)
            }
            DecodeValueErrorDetail::InvalidCount(ref count) => {
                write!(f, "Invalid count: {}", count)
            }
            DecodeValueErrorDetail::InvalidDataType(ref datatype) => {
                write!(f, "Invalid data type: {:?}", datatype)
            }
            DecodeValueErrorDetail::Overflow(ref value) => write!(f, "Overflow: {}", value),
        }
    }
}

impl Error for DecodeValueErrorDetail {}

#[derive(Debug)]
pub struct TagErrorKind {
    detail: TagErrorKindDetail,
    typename: &'static str,
}

impl TagErrorKind {
    pub fn cannot_find_tag<T: Tag>() -> Self {
        TagErrorKind {
            detail: TagErrorKindDetail::CannotFindTag,
            typename: T::typename(),
        }
    }
}

impl fmt::Display for TagErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.detail {
            TagErrorKindDetail::CannotConstructTag => {
                write!(f, "Cannot construct the tag: {}", self.typename)
            }
            TagErrorKindDetail::CannotFindTag => {
                write!(f, "Cannot find the tag: {}", self.typename)
            }
        }
    }
}

impl Error for TagErrorKind {}

#[derive(Debug)]
enum TagErrorKindDetail {
    CannotConstructTag,
    CannotFindTag,
}
