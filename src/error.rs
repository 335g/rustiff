
use std::io;
use std::fmt::{
    self,
    Display,
};
use ifd::DataType;
use tag::{
    TagType,
    ImpossibleTag,
    AnyTag,
};
use image::{
    ImageHeaderBuildError,
    ConstructError,
};
use failure::{
    Context,
    Fail,
    Backtrace,
};

/// `Result` type for handling `DecodeError`.
pub type DecodeResult<T> = ::std::result::Result<T, DecodeError>;

/// 
#[derive(Debug, Eq, PartialEq, Fail)]
pub enum FileHeaderErrorKind {
    /// Tiff file header has 2 byte data at the beginning.
    /// This error occurs when there is no 2 byte data.
    #[fail(display = "Incorrect header: No Byte Order")]
    NoByteOrder,

    /// Tiff file header has 2 byte data at the beginning.
    /// 2 byte data should be b'II' or b'MM'.
    /// This error occurs when 2 byte data is incorrect data.
    #[fail(display = "Incorrect header: Incorrect Byte Order, byte_order(`{:?}`) must be `b`II`` or `b`MM``", byte_order)]
    IncorrectByteOrder {
        #[allow(missing_docs)]
        byte_order: [u8; 2]
    },

    /// There is `0x00 0x2A` data after data corresponding to byte order.
    /// This error occurs when there is no this 2 byte data.
    #[fail(display = "Incorrect header: No Version, version must be 42.")]
    NoVersion,

    /// There is `0x00 0x2A` data after data corresponding to byte order.
    /// This error occurs when 2 byte data is not equal 42.
    #[fail(display = "Incorrect header: Incorrect Version")]
    IncorrectVersion { 
        #[allow(missing_docs)]
        version: u16
    },
    
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
    CannotFindTheTag { 
        /// The tag to be used.
        tag: AnyTag,
    },
    
    /// This error occurs when `datatype` & `count` used in the function of `tag::TagType::decode` 
    /// don't correspond to parsing `tag::TagType::Value`.
    ///
    /// All tag type implements `TagType` and have the `tag::TagType::Value` types.
    #[fail(display = "({}) doesn't support this datatype({:?}) and count({})", tag, datatype, count)]
    UnsupportedDataTypeAndCount { 
        /// Specified tag.
        tag: AnyTag,

        /// Specified `DataType` in `tag::TagType::decode`.
        datatype: DataType, 

        /// Specified count in `tag::TagType::decode`.
        count: usize 
    },

    /// This error occurs when trying to read a different size from buffer size.
    ///
    /// Buffer size is `width * height * samples_per_pixel`.
    /// `decoder::Decoder` reads data from file for each strip in `decoder::read_byte_only_u8`
    /// or `decoder::read_byte_only_u16` or `decoder::read_byte_u8_or_u16`.
    #[fail(display = "want(calc from `width *  height * samples/pixel`): {}, got: {}", want, got)]
    IncorrectBufferSize { 
        /// Wanted size (= `width * height * samples_per_pixel`).
        want: usize, 
        
        /// Actualy got size.
        got: usize
    },

    /// This error occurs when `ImageHeaderBuilder` cannot build `ImageHeader`.
    ///
    /// For example, if `PhotometricInterpretation` and `BitsPerSample` and `SamplesPerPixel`
    /// are incompatible, an error occurs.
    #[fail(display = "{}", _0)]
    IncompatibleHeaderData(#[fail(cause)] ImageHeaderBuildError), 
    
    /// This error occurs when you try to use the tag that cannot be used.
    ///
    /// Specifically, when to use `IFD::insert` or `AnyTag::try_from`, etc...
    #[fail(display = "Impossible tag: {}", tag)]
    ImpossibleTag { 
        /// Specified tag.
        ///
        /// This can be downcasted by `failure::Fail::downcast_ref` to the tag.
        tag: Box<dyn Fail>
    },

    /// This error occurs when construct fails.
    ///
    /// For example, `image::PhotometricInterpretation` must be between 0 
    /// (image::PhotometricInterpretation::WhiteIsZero) and 7 (image::PhotometricInterpretation::CIELab).
    /// This error occurs when you construct `image::PhotometricInterpretation` to use
    /// `image::PhotometricInterpretation::from_u16` with other than 0 to 7.
    #[fail(display = "{}", construct_error)]
    CannotConstruct { 
        /// Construct error
        ///
        /// This can be downcasted by `failure::Fail::downcast_ref` to the `image::ConstructError`.
        construct_error: Box<dyn Fail>
    },
}

/// Error type for decoding.
#[allow(missing_docs)]
#[derive(Debug)]
pub struct DecodeError {
    inner: Context<DecodeErrorKind>,
}

#[allow(missing_docs)]
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

#[allow(missing_docs)]
impl DecodeError {
    #[inline]
    fn new(kind: DecodeErrorKind) -> DecodeError {
        DecodeError { inner: Context::new(kind) }
    }
    
    pub fn kind(&self) -> &DecodeErrorKind {
        self.inner.get_context()
    }
}

#[allow(missing_docs)]
impl From<io::Error> for DecodeError {
    fn from(err: io::Error) -> DecodeError {
        DecodeError::new(DecodeErrorKind::Io(err)) 
    }
}

#[allow(missing_docs)]
impl From<FileHeaderErrorKind> for DecodeError {
    fn from(kind: FileHeaderErrorKind) -> DecodeError {
        DecodeError::new(DecodeErrorKind::IncorrectFileHeader(kind))
    }
}

#[allow(missing_docs)]
impl From<ImageHeaderBuildError> for DecodeError {
    fn from(err: ImageHeaderBuildError) -> DecodeError {
        DecodeError::new(DecodeErrorKind::IncompatibleHeaderData(err))
    }
}

#[allow(missing_docs)]
impl From<DecodeErrorKind> for DecodeError {
    fn from(kind: DecodeErrorKind) -> DecodeError {
        DecodeError { inner: Context::new(kind) }
    }
}

#[allow(missing_docs)]
impl<T> From<ImpossibleTag<T>> for DecodeError where T: TagType {
    fn from(tag: ImpossibleTag<T>) -> DecodeError {
        DecodeError::from(DecodeErrorKind::ImpossibleTag { tag: Box::new(tag.tag()) })
    }
}

#[allow(missing_docs)]
impl<T> From<ConstructError<T>> for DecodeError where T: TagType {
    fn from(err: ConstructError<T>) -> DecodeError {
        DecodeError::from(DecodeErrorKind::CannotConstruct { construct_error: Box::new(err) })
    }
}
