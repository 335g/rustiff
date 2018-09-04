
// tmp
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use ifd::{
    DataType,
    Entry,
};
use tag::{
    self,
    AnyTag,
};
use image::{
    PhotometricInterpretation,
    BitsPerSample,
    BitsPerSampleError,
    ImageHeaderError,
};

use std::io;
use std::fmt::{
    self,
    Display,
};
use std::error::Error as StdError;
use failure::{
    Context,
    Fail,
    Backtrace,
};

pub type DecodeResult<T> = ::std::result::Result<T, DecodeError>;

#[derive(Debug, Fail)]
pub enum DecodeErrorKind {
    #[fail(display = "IO Error: {:?}", error)]
    IO { error: io::Error },

    #[fail(display = "Incorrect header: No Byte Order")]
    NoByteOrder,

    #[fail(display = "Incorrect header: No Version")]
    NoVersion,

    #[fail(display = "Incorrect header: No IFD address")]
    NoIFDAddress,

    #[fail(display = "No Image address")]
    NoImage,

    #[fail(display = "Can't find the tag ({:?})", tag)]
    CannotFindTheTag { tag: AnyTag },

    #[fail(display = "Unsupported IFD Entry ({})\n  reason: {}", entry, reason)]
    UnsupportedIFDEntry{ entry: Entry, reason: String },

    #[fail(display = "Tag ({:?}) does not support data: ({:?})", tag, data)]
    UnsupportedMultipleData { tag: AnyTag, data: Vec<u32> },

    #[fail(display = "Tag ({:?}) does not support data: ({:?})", tag, data)]
    UnsupportedData { tag: AnyTag, data: u32 },

    #[fail(display = "Calculated from width and height: {}, sum: {}", calc, sum)]
    IncorrectBufferSize { calc: usize, sum: usize },

    #[fail(display = "Incompatible Data ({:?}/{:?}", photometric_interpretation, bits_per_sample)]
    IncompatibleData { photometric_interpretation: PhotometricInterpretation, bits_per_sample: BitsPerSample },

    #[fail(display = "Tag ({:?}) requires data, but you dont got any data", tag)]
    NoData { tag: AnyTag },

    #[fail(display = "Tag ({:?}) requires only one value, but you got extra data: {:?}.", tag, data)]
    ExtraData { tag: AnyTag, data: Vec<u32> },

    #[fail(display = "Tag ({:?}) requires a value less than or equal to `u16::max_value()`, you got {:?}", tag, data)]
    OverflowU16Data { tag: AnyTag, data: u32 },
    
    #[fail(display = "Tag ({:?}) requires a value less than or equal to `u8::max_value()`, you got {:?}", tag, data)]
    OverflowU8Data { tag: AnyTag, data: u32 },
}

#[derive(Debug)]
pub struct DecodeError {
    inner: Context<DecodeErrorKind>,
}

impl Fail for DecodeError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl DecodeError {
    fn new(kind: DecodeErrorKind) -> DecodeError {
        DecodeError { inner: Context::new(kind) }
    }
    
    pub fn kind(&self) -> &DecodeErrorKind {
        self.inner.get_context()
    }
}

impl From<io::Error> for DecodeError {
    fn from(err: io::Error) -> DecodeError {
        DecodeError::new(DecodeErrorKind::IO { error: err })
    }
}

impl From<BitsPerSampleError> for DecodeError {
    fn from(err: BitsPerSampleError) -> DecodeError {
        let values = err.values();
        let kind = DecodeErrorKind::UnsupportedMultipleData { 
            tag: AnyTag::BitsPerSample,
            data: err.values().iter().map(|x| *x as u32).collect::<_>()
        };

        DecodeError::new(kind)
    }
}

impl From<ImageHeaderError> for DecodeError {
    fn from(err: ImageHeaderError) -> DecodeError {
        let (photometric_interpretation, bits_per_sample) = match err {
            ImageHeaderError::IncompatibleData { photometric_interpretation, bits_per_sample } => (photometric_interpretation, bits_per_sample)
        };
            
        DecodeError::new(DecodeErrorKind::IncompatibleData { photometric_interpretation, bits_per_sample })
    }
}

impl From<DecodeErrorKind> for DecodeError {
    fn from(kind: DecodeErrorKind) -> DecodeError {
        DecodeError { inner: Context::new(kind) }
    }
}

pub enum EncodeError {}
