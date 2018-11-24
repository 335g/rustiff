
use ifd::DataType;
use tag::{
    TagType,
    TagError,
    AnyTag,
};
use image::{
    PhotometricInterpretation,
    BitsPerSample,
    ImageHeaderBuildError,
    ConstructError,
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

/// `Result` type for handling `DecodeError`.
pub type DecodeResult<T> = ::std::result::Result<T, DecodeError>;

/// 
#[derive(Debug, Fail)]
pub enum FileHeaderErrorKind {
    /// Tiff file header has 2 byte data corresponding to byte order.
    /// This error occurs when there is no correct data.
    #[fail(display = "Incorrect header: Byte Order")]
    NoByteOrder,

    /// There is `0x00 0x2A` data after data corresponding to byte order.
    /// This error occurs when there is no this 2 byte data.
    #[fail(display = "Incorrect header: No Version")]
    NoVersion,
    
    /// There is 4 byte data corresponding to an address of Image File Directory (IFD).
    /// This error occurs when there is no this 4 byte data.
    #[fail(display = "Incorrect header: No IFD address")]
    NoIFDAddress,
}

/// error details when decoding.
#[derive(Debug, Fail)]
pub enum DecodeErrorKind {
    /// This error occurs when `io::Error` occurs.
    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] io::Error),
    
    /// This error occurs when file header is not correct.
    #[fail(display = "{}", _0)]
    IncorrectFileHeader(#[fail(cause)] FileHeaderErrorKind),
    
    /// This error occurs when the IFD doesn't have this tag.
    #[fail(display = "Can't find the ({})", tag)]
    CannotFindTheTag { tag: AnyTag },
    
    /// This error occurs when `image::BitsPerSample::new` constructs `image::BitsPerSample` 
    /// with incorrect values. 
    ///
    /// Values less than or equal to 16 are supported. It can be different for each samples. 
    /// For example, if there are three samples such as RGB, R is 8 and G is 16.
    #[fail(display = "({:?}) is incorrect data for tag::BitsPerSample", data)]
    UnsupportedBitsPerSample { data: Vec<u16> },

    /// This error occurs when The value obtained by the `decoder::Decoder::get_value` is
    /// not supported value. 
    ///
    /// For example, `image::PhotometricInterpretation` supports the values between 0 and 7.
    /// Therefore, when other values are obtained, an error occurs.
    #[fail(display = "({}) does not support data: ({:?})", tag, data)]
    UnsupportedData { tag: AnyTag, data: u32 },
    
    /// This error occurs when `datatype` & `count` used in the function of `TagType::decode` 
    /// don't correspond to parsing `TagType::Value`.
    ///
    /// All tag type implements `TagType` and have the `TagType::Value` types.
    #[fail(display = "({}) doesn't support this datatype({:?}) and count({})", tag, datatype, count)]
    UnsupportedDataTypeAndCount { tag: AnyTag, datatype: DataType, count: usize },


    #[fail(display = "({}) does not support data({:?}). reason: {:?}", tag, data, reason)]
    UnsupportedUnitData { tag: AnyTag, data: u32, reason: &'static str },

    #[fail(display = "({}) does not support data({:?}). reason: {:?}", tag, data, reason)]
    UnsupportedMultipleData { tag: AnyTag, data: Vec<u32>, reason: &'static str },

    /// This error occurs when trying to read a different size from buffer size.
    ///
    /// Buffer size is `width * height * samples_per_pixel`.
    /// `decoder::Decoder` reads data from file for each strip in `decoder::read_byte_only_u8`
    /// or `decoder::read_byte_only_u16` or `decoder::read_byte_u8_or_u16`.
    #[fail(display = "want(calc from `width *  height * samples/pixel`): {}, got: {}", want, got)]
    IncorrectBufferSize { want: usize, got: usize },

    /// This error occurs when `ImageHeaderBuilder` cannot build `ImageHeader`.
    ///
    /// For example, if `PhotometricInterpretation` and `BitsPerSample` and `SamplesPerPixel`
    /// are incompatible, an error occurs.
    #[fail(display = "{}", _0)]
    IncompatibleHeaderData(#[fail(cause)] ImageHeaderBuildError), 

    /// 
    #[fail(display = "cannot use the tag: {:?}", description)]
    CannotUseTheTag { description: String }
}

/// Error type for decoding.
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
    #[inline]
    fn new(kind: DecodeErrorKind) -> DecodeError {
        DecodeError { inner: Context::new(kind) }
    }
    
    pub fn kind(&self) -> &DecodeErrorKind {
        self.inner.get_context()
    }
}

impl From<io::Error> for DecodeError {
    fn from(err: io::Error) -> DecodeError {
        DecodeError::new(DecodeErrorKind::Io(err)) 
    }
}

impl From<FileHeaderErrorKind> for DecodeError {
    fn from(kind: FileHeaderErrorKind) -> DecodeError {
        DecodeError::new(DecodeErrorKind::IncorrectFileHeader(kind))
    }
}

impl From<ImageHeaderBuildError> for DecodeError {
    fn from(err: ImageHeaderBuildError) -> DecodeError {
        DecodeError::new(DecodeErrorKind::IncompatibleHeaderData(err))
    }
}

impl From<DecodeErrorKind> for DecodeError {
    fn from(kind: DecodeErrorKind) -> DecodeError {
        DecodeError { inner: Context::new(kind) }
    }
}

impl<T> From<TagError<T>> for DecodeError where T: TagType {
    fn from(err: TagError<T>) -> DecodeError {
        let tag = err.tag();
        let description = format!("{:?}", tag);
        DecodeError::from(DecodeErrorKind::CannotUseTheTag { description: description })
    }
}

impl<T> From<ConstructError<T>> for DecodeError where T: TagType {
    fn from(err: ConstructError<T>) -> DecodeError {
        unimplemented!()
    }
}
