
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum PhotometricInterpretation {
    WhiteIsZero = 0,
    BlackIsZero = 1,
    RGB = 2,
    Palette = 3,
    TransparencyMask = 4,
    CMYK = 5,
}

impl Default for PhotometricInterpretation {
    fn default() -> Self {
        PhotometricInterpretation::WhiteIsZero
    }
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum Compression {
    No = 1,
    Huffman = 2,
    Fax3 = 3,
    Fax4 = 4,
    LZW = 5,
    JPEG = 6,
    PackBits = 0x8005,
}

impl Default for Compression {
    fn default() -> Self {
        Compression::No
    }
}

#[derive(Debug)]
pub struct Info {
    width: u32,
    height: u32,
    bits_per_sample: Vec<u8>,
    photometric_interpretation: PhotometricInterpretation,
    compression: Compression,
}

#[derive(Debug)]
pub struct Image {
    info: Info,
    data: Vec<u8>,
}

impl Image {
    pub fn new(info: Info, data: Vec<u8>) -> Self {
        Image {
            info: info,
            data: data,
        }
    }
}
