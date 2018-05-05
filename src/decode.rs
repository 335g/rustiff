
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use error::Result;
use byte::{
    Endian,
    EndianReadExt,
};

use std::{
    io::{
        self,
        Read,
        Seek,
    },
};

#[derive(Debug)]
pub struct Decoder<R> {
    reader: R,
    endian: Endian,
}

impl<R> Decoder<R> where R: Read + Seek {
    pub fn new(mut reader: R) -> Result<Decoder<R>> {
        let mut byte_order = [0u8; 2];
        reader.read(&mut byte_order)?;
        let endian = match &byte_order {
            b"II" => Endian::Little,
            b"MM" => Endian::Big,
            _ => bail!("Not tiff"),
        };

        if reader.read_u16(&endian)? != 42 {
            bail!("Not tiff");
        }

        let decoder = Decoder {
            reader: reader,
            endian: endian,
        };

        Ok(decoder)
    }
}


