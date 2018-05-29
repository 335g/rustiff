
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use byte::{
    EndianReader,
    StrictReader,
};

use error::{
    Result,
    Error,
    DecodeError,
};

#[derive(Debug, Clone, Copy)]
pub enum PhotometricInterpretation {
    WhiteIsZero,
    RGB,
    CMYK,
}

impl PhotometricInterpretation {
    pub fn from_u16(n: u16) -> Result<PhotometricInterpretation> {
        use self::PhotometricInterpretation::*;

        match n {
            0 => Ok(WhiteIsZero),
            2 => Ok(RGB),
            5 => Ok(CMYK),
            n => Err(Error::from(DecodeError::UnsupportedU16{ data: n })),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Compression {
    No,
}

impl Compression {
    pub fn from_u16(n: u16) -> Result<Compression> {
        use self::Compression::*;

        match n {
            1 => Ok(No),
            n => Err(Error::from(DecodeError::UnsupportedU16{ data: n })),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BitsPerSample {
    bits: Vec<u8>,
}

impl BitsPerSample {
    pub fn one(n: u8) -> BitsPerSample {
        BitsPerSample { bits: vec![n] }
    }

    pub fn three(xs: [u8; 3]) -> BitsPerSample {
        BitsPerSample { bits: vec![xs[0], xs[1], xs[2]] }
    }

    pub fn four(xs: [u8; 4]) -> BitsPerSample {
        BitsPerSample { bits: vec![xs[0], xs[1], xs[2], xs[3]] }
    }
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
        bits_per_sample: BitsPerSample) -> ImageHeader {

        ImageHeader {
            width: width,
            height: height,
            compression: compression,
            photometric_interpretation: interpretation,
            bits_per_sample: bits_per_sample,
        }
    }
}

pub enum ImageData {
    U8(Vec<u8>),
    U16(Vec<u16>),
}

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
}

