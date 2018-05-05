
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use byteorder::{
    LittleEndian,
    BigEndian,
    ReadBytesExt,
};
use error::Result;

use std::{
    io::{
        self,
        Read,
        Seek,
    },
};

#[derive(Debug)]
pub enum Endian {
    Big,
    Little,
}

pub trait EndianReadExt: Read {
    fn read_u16(&mut self, byte_order: &Endian) -> Result<u16> {
        let value = match *byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_u16::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_u16::<LittleEndian>(self),
        }?;

        Ok(value)
    }

    fn read_u32(&mut self, byte_order: &Endian) -> Result<u32> {
        let value = match *byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_u32::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_u32::<LittleEndian>(self),
        }?;

        Ok(value)
    }
}

impl<R: Read> EndianReadExt for R {}

pub trait ReadExt: Read {
    fn read_2byte(&mut self) -> Result<[u8; 2]> {
        let mut val = [0u8; 2];
        self.read_exact(&mut val)?;

        Ok(val)
    }

    fn read_4byte(&mut self) -> Result<[u8; 4]> {
        let mut val = [0u8; 4];
        self.read_exact(&mut val)?;

        Ok(val)
    }
}

impl<R: Read> ReadExt for R {}

pub trait SeekExt: Seek {
    // jump memory address.
    fn goto(&mut self, x: u64) -> Result<()> {
        self.seek(io::SeekFrom::Start(x))?;
        Ok(())
    }
}

impl<S: Seek> SeekExt for S {}

