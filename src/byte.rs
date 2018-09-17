
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
        Cursor,
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

    fn read_u16(&mut self, byte_order: Endian) -> io::Result<u16> {
        match byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_u16::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_u16::<LittleEndian>(self),
        }
    }

    fn read_u32(&mut self, byte_order: Endian) -> io::Result<u32> {
        match byte_order {
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

#[derive(Debug)]
pub struct LZWReader(Cursor<Vec<u8>>);

impl LZWReader {
    pub fn new<R>(reader: &mut R, compressed_len: usize) -> io::Result<(LZWReader, usize)> where R: Read {
        let mut compressed = vec![0; compressed_len as usize];
        reader.read_exact(&mut compressed)?;
        let mut uncompressed = vec![];
        let mut decoder = ::lzw::DecoderEarlyChange::new(::lzw::MsbReader::new(), 8);
        let mut read = 0;
        while read < compressed_len {
            let (len, bytes) = decoder.decode_bytes(&compressed[read..])?;
            read += len;
            uncompressed.extend_from_slice(bytes);
        }

        let bytes = uncompressed.len();
        let reader = LZWReader(io::Cursor::new(uncompressed));

        Ok((reader, bytes))
    }
}

impl Read for LZWReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

