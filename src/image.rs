
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use failure::Fail;

use byte::{
    EndianReader,
    StrictReader,
};
use error::{
    DecodeError,
    DecodeErrorKind,
};
use tag::TagKind;

use std::{
    fmt::Debug,
};

#[derive(Debug, Clone, Copy)]
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
            n => Err(DecodeError::from(DecodeErrorKind::UnsupportedData{ tag: TagKind::PhotometricInterpretation, data: n as u32 })),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Compression {
    No,
    LZW,
}

impl Compression {
    pub fn from_u16(n: u16) -> Result<Compression, DecodeError> {
        match n {
            1 => Ok(Compression::No),
            5 => Ok(Compression::LZW),
            n => Err(DecodeError::from(DecodeErrorKind::UnsupportedData{ tag: TagKind::Compression, data: n as u32 })),
        }
    }
}

#[derive(Debug, Fail)]
pub enum BitsPerSampleError {
    #[fail(display = "Invalid values: {:?}", values)]
    InvalidValues { values: Vec<u8> }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BitsPerSample {
    U8_3,
    U8_4,
    U16_3,
    U16_4,
    N(u8),
}

impl BitsPerSample {
    pub fn new<T: AsRef<[u8]>>(values: T) -> Result<BitsPerSample, BitsPerSampleError> {
        match values.as_ref() {
            [8, 8, 8] => Ok(BitsPerSample::U8_3),
            [8, 8, 8, 8] => Ok(BitsPerSample::U8_4),
            [16, 16, 16] => Ok(BitsPerSample::U16_3),
            [16, 16, 16, 16] => Ok(BitsPerSample::U16_4),
            [n] if *n <= 8 => Ok(BitsPerSample::N(*n)),
            _ => Err(BitsPerSampleError::InvalidValues { values: values.as_ref().to_vec() }),
        }
    }

    pub fn len(&self) -> usize {
        match *self {
            BitsPerSample::U8_3 | BitsPerSample::U16_3 => 3,
            BitsPerSample::U8_4 | BitsPerSample::U16_4 => 4,
            BitsPerSample::N(_) => 1,
        }
    }

    pub fn max_values(&self) -> Vec<u16> {
        let x = u8::max_value() as u16;
        let y = u16::max_value();

        match *self {
            BitsPerSample::U8_3 => vec![x, x, x],
            BitsPerSample::U8_4 => vec![x, x, x, x],
            BitsPerSample::U16_3 => vec![y, y, y],
            BitsPerSample::U16_4 => vec![y, y, y, y],
            BitsPerSample::N(n) => vec![2u16.pow(n as u32)]
        }
    }
}

//#[derive(Debug, Clone, PartialEq, Eq)]
//pub struct BitsPerSample(Vec<u8>);
//
//impl BitsPerSample {
//    pub fn new<T: AsRef<[u8]>>(values: T) -> Result<BitsPerSample, BitsPerSampleError> {
//        BitsPerSample(value.as_ref().to_vec())
//    }
//    
//    #[inline]
//    pub fn bits(&self) -> &[u8] {
//        &self.0
//    }
//    
//    #[inline]
//    pub fn len(&self) -> usize {
//        self.bits().len()
//    }
//    
//    #[inline]
//    pub fn max_values<'a>(&'a self) -> impl Iterator<Item=u16> + 'a {
//        self.bits().iter().map(|x| 2u16.pow(*x as u32) - 1)
//    }
//}

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
        bits_per_sample: BitsPerSample) -> ImageHeader {

        ImageHeader {
            width: width,
            height: height,
            compression: compression,
            photometric_interpretation: interpretation,
            bits_per_sample: bits_per_sample,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bits_per_sample(&self) -> &BitsPerSample {
        &self.bits_per_sample
    }

    pub fn compression(&self) -> Compression {
        self.compression
    }
}

pub struct Image {
    header: ImageHeader,
    data: Vec<u8>,
}

impl Image {
    pub fn new<D: AsRef<[u8]>>(header: ImageHeader, data: D) -> Image {
        Image {
            header: header,
            data: data.as_ref().to_vec()
        }
    }
}

