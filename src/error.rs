
use ifd::DataType;
use tag::{
    TagType,
    AnyTag,
};
use image::{
    PhotometricInterpretation,
    BitsPerSample,
    ImageHeaderError,
};
use std::io;
use std::fmt::{
    self,
    Display,
};
use failure::{
    Context,
    Fail,
    Backtrace,
};

///
#[derive(Debug, Fail)]
pub enum TagError<T: TagType> {
    /// 
    #[fail(display = "Unsupported tag: {:?}", tag)]
    UnsupportedTag { tag: T },
}

/// `Result` type for handling `DecodeError`.
pub type DecodeResult<T> = ::std::result::Result<T, DecodeError>;

/// 
#[derive(Debug, Fail)]
pub enum HeaderErrorKind {
    /// Tiff file header has 2 byte data corresponding to byte order.
    /// This error occurs when there is no correct data.
    #[fail(display = "Incorrect Byte Order")]
    NoByteOrder,

    /// There is `0x00h 0x2Ah` data after data corresponding to byte order.
    /// This error occurs when there is no this 2 byte data.
    #[fail(display = "No Version")]
    NoVersion,
    
    /// There is 4 byte data corresponding to an address of Image File Directory (IFD).
    /// This error occurs when there is no this 4 byte data.
    #[fail(display = "No IFD address")]
    NoIFDAddress,
}

/// error details when decoding.
#[derive(Debug, Fail)]
pub enum DecodeErrorKind {
    /// This error occurs when `io::Error` occurs.
    #[fail(display = "IO Error: {:?}", error)]
    Io { error: io::Error },
    
    /// Tiff file firstly has header part.
    /// This error occurs when header part is not correct.
    #[fail(display = "Incorrect header: {:?}", detail)]
    IncorrectHeader { detail: HeaderErrorKind },
    
    /// This error occurs when the IFD doesn't have this tag.
    #[fail(display = "Can't find the tag ({:?})", tag)]
    CannotFindTheTag { tag: AnyTag },
    
    /// This error occurs when `image::BitsPerSample::new` constructs `image::BitsPerSample` 
    /// with incorrect values. Incorrect values are all 8 or all 16.
    ///
    #[fail(display = "({:?}) is incorrect data for tag::BitsPerSample", data)]
    IncorrectBitsPerSample { data: Vec<u16> },

    /// This error occurs when The value obtained by the `decoder::Decoder::get_value` is
    /// not supported value. 
    ///
    /// For example, `image::PhotometricInterpretation` supports the values between 0 and 7.
    /// Therefore, when other values are obtained, an error occurs.
    #[fail(display = "Tag ({:?}) does not support data: ({:?})", tag, data)]
    UnsupportedData { tag: AnyTag, data: u32 },
    
    /// `decoder::Decoder` reads data from file for each strip in `decoder::read_byte_detail_u8`
    /// or `decoder::read_byte_detail_u16`. This errors occur when trying to read a larger value
    /// than prepared buffer size.
    #[fail(display = "Calculated from width and height: {}, sum: {}", calc, sum)]
    IncorrectBufferSize { calc: usize, sum: usize },

    /// This error occurs when `PhotometricInterpretation` and `BitsPerSample` and `SamplesPerPixel`
    /// are not compatible. Especially, when extracting image information (with `decode::image` &
    /// `decode::image_with`) and extracting image header information (with `decode::header` &
    /// `decode::header_with`). Just getting value by `decoder::get_value` will not result
    /// in an error.
    #[fail(display = "{:?}", err)]
    IncompatibleData { err: ImageHeaderError },

    /// All tag type implements `TagType` and have the `TagType::Value` types. This error occurs
    /// when `datatype` & `count` used in the function of `TagType::decode` don't correspond to 
    /// parsing `TagType::Value`.
    #[fail(display = "Tag ({:?}) doesn't support this datatype/count : {:?}/{}", tag, datatype, count)]
    NoSupportDataType { tag: AnyTag, datatype: DataType, count: usize },

    ///
    #[fail(display = "Unsupported tag: {:?}", boxed_tag)]
    UnsupportedTag { boxed_tag: Box<dyn Fail> },
}

/// Erro type for decoding.
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
        DecodeError::new(DecodeErrorKind::Io { error: err })
    }
}

impl From<HeaderErrorKind> for DecodeError {
    fn from(err: HeaderErrorKind) -> DecodeError {
        DecodeError::new(DecodeErrorKind::IncorrectHeader { detail: err })
    }
}

impl From<ImageHeaderError> for DecodeError {
    fn from(err: ImageHeaderError) -> DecodeError {
        DecodeError::new(DecodeErrorKind::IncompatibleData { err: err })
    }
}

impl From<DecodeErrorKind> for DecodeError {
    fn from(kind: DecodeErrorKind) -> DecodeError {
        DecodeError { inner: Context::new(kind) }
    }
}

impl<T> From<TagError<T>> for DecodeError where T: TagType {
    fn from(err: TagError<T>) -> DecodeError {
        let tag = match err {
            TagError::UnsupportedTag { tag: tag } => tag,
        };
        DecodeError::new(DecodeErrorKind::UnsupportedTag { boxed_tag: Box::new(tag) })
    }
}

