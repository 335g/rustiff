
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

#[derive(Debug, Clone, Copy)]
pub enum Endian {
    Big,
    Little,
}

pub trait EndianReadExt: Read {
    fn read_u8(&mut self) -> Result<u8> {
        let value = <Self as ReadBytesExt>::read_u8(self)?;
        Ok(value)
    }

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

pub trait EndianReader<R: Read + Seek>: Sized {
    fn new(reader: R, endian: Endian, length: usize) -> Result<(usize, Self)>;
}

#[derive(Debug)]
pub struct StrictReader<R> {
    reader: R,
    endian: Endian,
}

impl<R> EndianReader<R> for StrictReader<R> where R: Read + Seek {
    fn new(reader: R, endian: Endian, length: usize) -> Result<(usize, Self)> {
        let r = StrictReader {
            reader: reader,
            endian: endian,
        };

        Ok((length, r))
    }
}

