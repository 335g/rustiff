use std::io;
use std::marker::PhantomData;
use std::{convert::From, num::TryFromIntError};
use std::{fmt, fs::File};

use crate::dir::DataType;
use crate::tag::{AnyTag, Tag};

pub type DecodeResult<T> = std::result::Result<T, DecodeError>;

#[derive(Debug)]
pub struct DecodeError(DecodeErrorKind);

impl DecodeError {
    pub(crate) fn new(kind: DecodeErrorKind) -> DecodeError {
        DecodeError(kind)
    }

    pub fn kind(&self) -> &DecodeErrorKind {
        &self.0
    }

    pub fn into_kind(self) -> DecodeErrorKind {
        self.0
    }

    pub fn is_io_error(&self) -> bool {
        match self.0 {
            DecodeErrorKind::Io(_) => true,
            _ => false,
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.0 {
            DecodeErrorKind::Io(ref err) => Some(err),
            DecodeErrorKind::FileHeader(ref err) => Some(err),
            DecodeErrorKind::Tag(ref err) => Some(err),
            DecodeErrorKind::Value(ref err) => Some(err),
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = match self.kind() {
            DecodeErrorKind::Io(err) => format!("io error: {:?}", err),
            DecodeErrorKind::FileHeader(err) => format!("file header error: {:?}", err),
            DecodeErrorKind::Tag(err) => format!("tag error: {:?}", err),
            DecodeErrorKind::Value(err) => format!("value error: {:?}", err),
        };

        write!(f, "{}", desc)
    }
}

impl From<io::Error> for DecodeError {
    fn from(err: io::Error) -> DecodeError {
        DecodeError::new(DecodeErrorKind::Io(err))
    }
}

impl From<FileHeaderError> for DecodeError {
    fn from(detail: FileHeaderError) -> DecodeError {
        DecodeError::new(DecodeErrorKind::FileHeader(detail))
    }
}

impl From<DecodeValueError> for DecodeError {
    fn from(detail: DecodeValueError) -> Self {
        DecodeError::new(DecodeErrorKind::Value(detail))
    }
}

impl From<std::num::TryFromIntError> for DecodeError {
    fn from(err: std::num::TryFromIntError) -> DecodeError {
        DecodeError::new(DecodeErrorKind::Value(DecodeValueError::Overflow(err)))
    }
}

impl From<TagError> for DecodeError {
    fn from(err: TagError) -> DecodeError {
        DecodeError::new(DecodeErrorKind::Tag(err))
    }
}

#[derive(Debug)]
pub enum DecodeErrorKind {
    Io(io::Error),
    FileHeader(FileHeaderError),
    Value(DecodeValueError),
    Tag(TagError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileHeaderError {
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

impl fmt::Display for FileHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = match self {
            FileHeaderError::NoByteOrder => "no byte order".to_string(),
            FileHeaderError::InvalidByteOrder { byte_order: x } => {
                format!("invalid byte order: {:?}", x)
            }
            FileHeaderError::NoVersion => "no version".to_string(),
            FileHeaderError::InvalidVersion { version: x } => format!("invalid version: {}", x),
            FileHeaderError::NoIFDAddress => "no ifd address".to_string(),
        };

        write!(f, "{}", desc)
    }
}

impl std::error::Error for FileHeaderError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeValueError {
    InvalidValue(Vec<u32>),

    InvalidCount(u32),

    InvalidDataType(DataType),

    Overflow(std::num::TryFromIntError),

    NoValueThatShouldBe,
}

impl fmt::Display for DecodeValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = match self {
            DecodeValueError::InvalidValue(x) => format!("invalid value: {:?}", x),
            DecodeValueError::InvalidCount(x) => format!("invalid count: {}", x),
            DecodeValueError::InvalidDataType(x) => format!("invalid data type: {:?}", x),
            DecodeValueError::Overflow(x) => format!("overflow: {:?}", x),
            DecodeValueError::NoValueThatShouldBe => "no value that should be".to_string(),
        };

        write!(f, "{}", desc)
    }
}

impl std::error::Error for DecodeValueError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagError {
    tag: AnyTag,
    kind: TagErrorKind,
}

impl TagError {
    pub(crate) fn new(tag: AnyTag, kind: TagErrorKind) -> TagError {
        TagError { tag, kind }
    }
}

impl fmt::Display for TagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error related to tag({}), reason: {}",
            self.tag, self.kind
        )
    }
}

impl std::error::Error for TagError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagErrorKind {
    CannotFindTag,
}

impl fmt::Display for TagErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = match self {
            TagErrorKind::CannotFindTag => "cannot find the tag",
        };

        write!(f, "{}", desc)
    }
}
