
// tmp
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use ifd::{
    DataType,
    Entry,
};
use tag::TagKind;

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

    #[fail(display = "Can't find the tag ({})", tag)]
    CannotFindTheTag { tag: TagKind },

    #[fail(display = "Unsupported IFD Entry ({})\n  reason: {}", entry, reason)]
    UnsupportedIFDEntry{ entry: Entry, reason: String },

    #[fail(display = "u32: ({}) which is the value of tag: ({}) overflows more than u8::max", tag, value)]
    OverflowU8Value { tag: TagKind, value: u32 },

    #[fail(display = "u32 ({}) which is the value of tag: ({}) overflows more than u16::max", tag, value)]
    OverflowU16Value { tag: TagKind, value: u32 },
    
    #[fail(display = "samples: {} != length of `bits_per_sample`: {:?}", samples, bits_per_sample)]
    IncorrectNumberOfSamples { samples: u8, bits_per_sample: Vec<u8> },

    #[fail(display = "tag: ({}) does not support data: ({})", tag, data)]
    UnsupportedData { tag: TagKind, data: u32 },

    #[fail(display = "calculated from width and height: {}, sum: {}", calc, sum)]
    IncorrectBufferSize { calc: usize, sum: usize },

    #[fail(display = "TagKind::BitsPerSample gets incorrect values: {:?}", values)]
    IncorrectBitsPerSample { values: Vec<u8> },
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

impl From<DecodeErrorKind> for DecodeError {
    fn from(kind: DecodeErrorKind) -> DecodeError {
        DecodeError { inner: Context::new(kind) }
    }
}

pub enum EncodeError {}
