
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
use ifd::Tag;

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
    pub fn from_u16(n: u16) -> Result<PhotometricInterpretation> {
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
            n => Err(Error::from(DecodeError::UnsupportedData{ tag: Tag::PhotometricInterpretation, data: n as u32 })),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Compression {
    No,
    LZW,
}

impl Compression {
    pub fn from_u16(n: u16) -> Result<Compression> {
        use self::Compression::*;

        match n {
            1 => Ok(No),
            5 => Ok(LZW),
            n => Err(Error::from(DecodeError::UnsupportedData{ tag: Tag::Compression, data: n as u32 })),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitsPerSample(Vec<u8>);

impl BitsPerSample {
    pub fn new<T: AsRef<[u8]>>(value: T) -> BitsPerSample {
        BitsPerSample(value.as_ref().to_vec())
    }

    pub fn all_bits(&self) -> &[u8] {
        &self.0
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

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bits_per_sample(&self) -> &Vec<u8> {
        &self.bits_per_sample.0
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

