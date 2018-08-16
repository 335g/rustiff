
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
        Seek,
    },
};

#[derive(Debug, Clone, Copy)]
pub enum Endian {
    Big,
    Little,
}

pub trait EndianReadExt: Read {
    fn read_u8(&mut self) -> io::Result<u8> {
        <Self as ReadBytesExt>::read_u8(self)
    }

    fn read_u16(&mut self, byte_order: &Endian) -> io::Result<u16> {
        match *byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_u16::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_u16::<LittleEndian>(self),
        }
    }

    fn read_u32(&mut self, byte_order: &Endian) -> io::Result<u32> {
        match *byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_u32::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_u32::<LittleEndian>(self),
        }
    }
}

impl<R: Read> EndianReadExt for R {}

pub trait ReadExt: Read {
    fn read_2byte(&mut self) -> io::Result<[u8; 2]> {
        let mut val = [0u8; 2];
        let _ = self.read_exact(&mut val)?;
        Ok(val)
    }

    fn read_4byte(&mut self) -> io::Result<[u8; 4]> {
        let mut val = [0u8; 4];
        let _ = self.read_exact(&mut val)?;
        Ok(val)
    }
}

impl<R: Read> ReadExt for R {}

pub trait SeekExt: Seek {
    // jump memory address.
    fn goto(&mut self, x: u64) -> io::Result<()> {
        self.seek(io::SeekFrom::Start(x))
            .map(|_| ())
    }
}

impl<S: Seek> SeekExt for S {}

pub trait EndianReader<R: Read + Seek>: Read {}

#[derive(Debug)]
pub struct StrictReader<R> {
    reader: R,
    endian: Endian,
}

impl<R> StrictReader<R> where R: Read + Seek {
    pub fn new(reader: R, endian: Endian) -> StrictReader<R> {
        StrictReader {
            reader: reader,
            endian: endian,
        }
    }
}

impl<R> Read for StrictReader<R> where R: Read {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}


