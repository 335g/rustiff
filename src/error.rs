use crate::{data::DataType, element::Element, tag::Tag};
use std::{io, marker::PhantomData};

pub type DecodeResult<T> = Result<T, DecodeError>;

#[derive(Debug)]
pub struct DecodeError(DecodeErrorKind);

impl DecodeError {
    pub(crate) fn new(kind: DecodeErrorKind) -> Self {
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

#[derive(Debug)]
pub enum DecodeErrorKind {
    Io(io::Error),
    FileHeader(FileHeaderError),
    UnsupportedDataType(u16),
    Decoding(DecodingError),
}

impl From<io::Error> for DecodeError {
    fn from(err: io::Error) -> Self {
        DecodeError::new(DecodeErrorKind::Io(err))
    }
}

impl From<FileHeaderError> for DecodeError {
    fn from(err: FileHeaderError) -> Self {
        DecodeError::new(DecodeErrorKind::FileHeader(err))
    }
}

impl From<DecodingError> for DecodeError {
    fn from(err: DecodingError) -> Self {
        DecodeError::new(DecodeErrorKind::Decoding(err))
    }
}

#[derive(Debug)]
pub struct TagError<T: Tag> {
    ghost: PhantomData<fn() -> T>,
    kind: TagErrorKind,
}

impl<T: Tag> TagError<T> {
    pub fn new(kind: TagErrorKind) -> Self {
        TagError {
            ghost: PhantomData,
            kind,
        }
    }

    pub fn kind(&self) -> &TagErrorKind {
        &self.kind
    }

    pub fn into_kind(self) -> TagErrorKind {
        self.kind
    }
}

#[derive(Debug)]
pub enum TagErrorKind {
    UnauthorizedTag { tag_ty: &'static str },
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

#[derive(Debug)]
pub enum DecodingError {
    Io(io::Error),
    InvalidDataCount(usize),
    InvalidValue(Element),
    InvalidDataType(DataType),
    Tag(TagErrorKind),
    NoExistShouldExist,
    OverCapacity,
}

impl From<io::Error> for DecodingError {
    fn from(err: io::Error) -> Self {
        DecodingError::Io(err)
    }
}
