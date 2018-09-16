
use error::{
    DecodeError,
    DecodeErrorKind,
};
use tag::AnyTag;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhotometricInterpretation {
    WhiteIsZero,
    BlackIsZero,
    RGB,
    Palette,
    TransparencyMask,
    CMYK,
    YCbCr,
    CIELab,
}

impl PhotometricInterpretation {
    pub fn from_u16(n: u16) -> Result<PhotometricInterpretation, DecodeError> {
        use self::PhotometricInterpretation::*;

        match n {
            0 => Ok(WhiteIsZero),
            1 => Ok(BlackIsZero),
            2 => Ok(RGB),
            3 => Ok(Palette),
            4 => Ok(TransparencyMask),
            5 => Ok(CMYK),
            6 => Ok(YCbCr),
            7 => Ok(CIELab),
            n => Err(DecodeError::from(DecodeErrorKind::UnsupportedData{ tag: AnyTag::PhotometricInterpretation, data: n as u32 })),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Compression {
    No,
    LZW,
}

impl Compression {
    pub fn from_u16(n: u16) -> Result<Compression, DecodeError> {
        match n {
            1 => Ok(Compression::No),
            5 => Ok(Compression::LZW),
            n => Err(DecodeError::from(DecodeErrorKind::UnsupportedData{ tag: AnyTag::Compression, data: n as u32 })),
        }
    }
}

#[derive(Debug, Fail)]
pub enum BitsPerSampleError {
    #[fail(display = "Invalid values: {:?}", values)]
    InvalidValues { values: Vec<u16> }
}

impl BitsPerSampleError {
    pub fn values(&self) -> &Vec<u16> {
        match self {
            BitsPerSampleError::InvalidValues { values } => values
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitsPerSample {
    U8_1,
    U8_3,
    U8_4,
    U16_1,
    U16_3,
    U16_4,
}

impl BitsPerSample {
    pub fn new<T: AsRef<[u16]>>(values: T) -> Result<BitsPerSample, BitsPerSampleError> {
        match values.as_ref() {
            [8] => Ok(BitsPerSample::U8_1),
            [8, 8, 8] => Ok(BitsPerSample::U8_3),
            [8, 8, 8, 8] => Ok(BitsPerSample::U8_4),
            [16] => Ok(BitsPerSample::U16_1),
            [16, 16, 16] => Ok(BitsPerSample::U16_3),
            [16, 16, 16, 16] => Ok(BitsPerSample::U16_4),
            _ => Err(BitsPerSampleError::InvalidValues { values: values.as_ref().to_vec() }),
        }
    }

    pub fn len(&self) -> usize {
        match *self {
            BitsPerSample::U8_1 | BitsPerSample::U16_1 => 1,
            BitsPerSample::U8_3 | BitsPerSample::U16_3 => 3,
            BitsPerSample::U8_4 | BitsPerSample::U16_4 => 4,
        }
    }

    pub fn max_value(&self) -> u16 {
        let x = u8::max_value() as u16;
        let y = u16::max_value();

        match *self {
            BitsPerSample::U8_1 | BitsPerSample::U8_3 | BitsPerSample::U8_4 => x,
            BitsPerSample::U16_1 | BitsPerSample::U16_3 | BitsPerSample::U16_4 => y,
        }
    }

    pub fn bits(&self) -> usize {
        match self {
            BitsPerSample::U8_1 | BitsPerSample::U8_3 | BitsPerSample::U8_4 => 8,
            BitsPerSample::U16_1 | BitsPerSample::U16_3 | BitsPerSample::U16_4 => 16
        }
    }
}

#[derive(Debug, Fail)]
pub enum ImageHeaderError {
    #[fail(display = "Incompatible data ({:?}/{:?}", photometric_interpretation, bits_per_sample)]
    IncompatibleData { photometric_interpretation: PhotometricInterpretation, bits_per_sample: BitsPerSample },
}

#[derive(Debug, Clone)]
pub struct ImageHeader {
    width: u32,
    height: u32,
    compression: Compression,
    photometric_interpretation: PhotometricInterpretation,
    bits_per_sample: BitsPerSample,
}

impl ImageHeader {
    pub fn new(
        width: u32, 
        height: u32, 
        compression: Compression, 
        interpretation: PhotometricInterpretation,
        bits_per_sample: BitsPerSample) -> Result<ImageHeader, ImageHeaderError>
    {
        if !is_valid_color_type(interpretation, bits_per_sample) {
            return Err(ImageHeaderError::IncompatibleData { 
                photometric_interpretation: interpretation, 
                bits_per_sample: bits_per_sample,
            });
        }

        let header = ImageHeader {
            width: width,
            height: height,
            compression: compression,
            photometric_interpretation: interpretation,
            bits_per_sample: bits_per_sample,
        };

        Ok(header)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bits_per_sample(&self) -> BitsPerSample {
        self.bits_per_sample
    }

    pub fn compression(&self) -> Compression {
        self.compression
    }

    pub fn photometric_interpretation(&self) -> PhotometricInterpretation {
        self.photometric_interpretation
    }
}

#[derive(Debug)]
pub enum ImageData { 
    U8(Vec<u8>),
    U16(Vec<u16>),
}

#[derive(Debug)]
pub struct Image {
    header: ImageHeader,
    data: ImageData,
}

impl Image {
    pub fn new(header: ImageHeader, data: ImageData) -> Image {
        Image {
            header: header,
            data: data,
        }
    }

    pub fn header(&self) -> &ImageHeader {
        &self.header
    }

    pub fn data(&self) -> &ImageData {
        &self.data
    }
}

#[inline]
fn is_valid_color_type(photometric_interpretation: PhotometricInterpretation, bits_per_sample: BitsPerSample) -> bool {
    use self::PhotometricInterpretation::*;
    use self::BitsPerSample::*;

    match (photometric_interpretation, bits_per_sample) {
        (RGB, U8_3) | 
        (RGB, U8_4) | 
        (RGB, U16_3) | 
        (RGB, U16_4) |
        (CMYK, U8_4) | 
        (CMYK, U16_4) |
        (BlackIsZero, U8_1) | 
        (BlackIsZero, U16_1) |
        (WhiteIsZero, U8_1) | 
        (WhiteIsZero, U16_1) => true,
        _ => false
    }
}
