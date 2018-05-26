
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use byte::{
    EndianReader,
    StrictReader,
};

#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum PhotometricInterpretation {
    WhiteIsZero = 0,
    RGB = 2,
    CMYK = 5,
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum Compression {
    No = 1,
}

pub enum Image {
    U8(Vec<u8>),
}


