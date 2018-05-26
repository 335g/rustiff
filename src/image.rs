
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use byte::{
    EndianReader,
    StrictReader,
};

#[derive(Debug, Clone, Copy)]
pub enum PhotometricInterpretation {
    WhiteIsZero,
    RGB,
    CMYK,
    Unknown(u16),
}

impl PhotometricInterpretation {
    pub fn from_u16(n: u16) -> PhotometricInterpretation {
        use self::PhotometricInterpretation::*;

        match n {
            0 => WhiteIsZero,
            2 => RGB,
            5 => CMYK,
            n => Unknown(n),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Compression {
    No,
    Unknown(u16),
}

impl Compression {
    pub fn from_u16(n: u16) -> Compression {
        use self::Compression::*;

        match n {
            1 => No,
            n => Unknown(n),
        }
    }
}

pub enum Image {
    U8(Vec<u8>),
}


