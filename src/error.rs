use std::io;
use std::marker::PhantomData;
use std::{convert::From, num::TryFromIntError};
use std::{fmt, fs::File};

use crate::data::DataType;
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

impl From<DecodingError> for DecodeError {
    fn from(detail: DecodingError) -> Self {
        DecodeError::new(DecodeErrorKind::Value(detail))
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
    Value(DecodingError),
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
pub enum DecodingError {
    /// A limit exists on the number of tags that can be supported.
    /// For example, if the image is uncompressed, ther value decoded by
    /// the `Compression` tag is 1, and if the image is compressed in LZW
    /// format, it is 5.
    /// This error occurs when the decoded value meets an unsupported value.
    UnsupportedValue(Vec<u16>),

    /// Values that implement `Decoded` have a limited number of data.
    /// For example, The value decoded by the `PhotometricInterpretation` tag
    /// is determined to be a single short(`u16`) value.
    /// This error occurs when the number of data is inconsistent.
    InvalidCount(u32),

    /// Values that implement `Decoded` have its own corresponding value type.
    /// For example, When decoding from `val::Byte`, which implements `Decoded`,
    /// `data::Entry.ty` should be `data::DataType::Byte`.
    /// This error occurs when the corresponding type is different.
    InvalidDataType(DataType),

    ///
    NoValueThatShouldBe,
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = match self {
            DecodingError::UnsupportedValue(x) => format!("invalid value: {:?}", x),
            DecodingError::InvalidCount(x) => format!("invalid count: {}", x),
            DecodingError::InvalidDataType(x) => format!("invalid data type: {:?}", x),
            DecodingError::NoValueThatShouldBe => "no value that should be".to_string(),
        };

        write!(f, "{}", desc)
    }
}

impl std::error::Error for DecodingError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagError {
    kind: TagErrorKind,
}

impl TagError {
    pub(crate) fn new(kind: TagErrorKind) -> TagError {
        TagError { kind }
    }
}

impl fmt::Display for TagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error related to tag, reason: {}",
            self.kind
        )
    }
}

impl std::error::Error for TagError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagErrorKind {
    UnauthorizedTag(String),
}

impl fmt::Display for TagErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = match self {
            TagErrorKind::UnauthorizedTag(x) => format!("`{}` tag is not autorized", x),
        };

        write!(f, "{}", desc)
    }
}
