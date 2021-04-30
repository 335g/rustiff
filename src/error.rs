
use std::fmt;
use std::io;
use crate::data::DataType;

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
            // DecodeErrorKind::Tag(ref err) => Some(err),
            // DecodeErrorKind::Value(ref err) => Some(err),
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = match self.kind() {
            DecodeErrorKind::Io(err) => format!("io error: {:?}", err),
            DecodeErrorKind::FileHeader(err) => format!("file header error: {:?}", err),
            // DecodeErrorKind::Tag(err) => format!("tag error: {:?}", err),
            // DecodeErrorKind::Value(err) => format!("value error: {:?}", err),
        };

        write!(f, "{}", desc)
    }
}

#[derive(Debug)]
pub enum DecodeErrorKind {
    Io(io::Error),
    FileHeader(FileHeaderError),

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

#[derive(Debug)]
pub enum DecodingError {
    ///
    UnsupportedValueForDataType(u16),

    ///
    UnsupportedShortValueForData {
        data: &'static str,
        value: Vec<u16>,
    },

    ///
    Element(DecodingElementError),


}

#[derive(Debug)]
pub enum DecodingElementError {
    Io(io::Error),

    ///
    NoMatchDataType {
        element: &'static str,
        datatype: DataType,
    }
}

impl From<io::Error> for DecodingElementError {
    fn from(err: io::Error) -> Self {
        DecodingElementError::Io(err)
    }
}




#[derive(Debug)]
pub struct EncodeError(EncodeErrorKind);

impl EncodeError {
    pub(crate) fn new(kind: EncodeErrorKind) -> EncodeError {
        EncodeError(kind)
    }

    pub fn kind(&self) -> &EncodeErrorKind {
        &self.0
    }

    pub fn into_kind(self) -> EncodeErrorKind {
        self.0
    }

    pub fn is_io_error(&self) -> bool {
        match self.0 {
            EncodeErrorKind::Io(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum EncodeErrorKind {
    Io(io::Error),

}

#[derive(Debug)]
pub enum EncodingError {
    
}