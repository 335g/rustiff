
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use byteorder::{
    LittleEndian,
    BigEndian,
    ReadBytesExt,
};

use std::{
    io::{
        self,
        Read,
    },
};

#[derive(Debug)]
pub enum Endian {
    Big,
    Little,
}

pub trait EndianReadExt: Read {
    #[inline]
    fn read_u16(&mut self, byte_order: &Endian) -> Result<u16, io::Error> {
        match *byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_u16::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_u16::<LittleEndian>(self),
        }
    }

    #[inline]
    fn read_u32(&mut self, byte_order: &Endian) -> Result<u32, io::Error> {
        match *byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_u32::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_u32::<LittleEndian>(self),
        }
    }
}

impl<R: Read> EndianReadExt for R {}

