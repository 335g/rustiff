
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use error::{
    Result,
    DecodeError,
    Error,
};
use byte::{
    Endian,
    EndianReadExt,
    ReadExt,
    SeekExt,
};
use ifd::{
    self,
    IFD,
    Entry,
};

use std::{
    io::{
        self,
        Read,
        Seek,
    },
    marker::PhantomData,
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
            b"II" => Ok(Endian::Little),
            b"MM" => Ok(Endian::Big),
            _ => Err(Error::from(DecodeError::IncorrectHeader{ reason: "byteorder".to_string() })),
        }?;

        if reader.read_u16(&endian)? != 42 {
            return Err(Error::from(DecodeError::IncorrectHeader{ reason: "Not 42".to_string() }));
        }

        let decoder = Decoder {
            reader: reader,
            endian: endian,
        };

        Ok(decoder)
    }

    pub fn ifds<'a>(&'a mut self) -> Result<IFDs<'a, R>> {
        match IFDs::new(&mut self.reader, self.endian) {
            Ok(ifds) => Ok(ifds),
            Err(e) => Err(Error::from(DecodeError::IncorrectHeader{ reason: "Not 5-8byte index".to_string() }))
        }
    }
}

pub struct IFDs<'a, R: 'a> {
    reader: &'a mut R,
    endian: Endian,
    next: u32,
}

impl<'a, R> IFDs<'a, R> where R: Read + Seek + 'a {
    pub fn new(reader: &'a mut R, endian: Endian) -> Result<IFDs<'a, R>> {
        reader.goto(4)?;
        let next = reader.read_u32(&endian)?;

        let x = IFDs {
            reader: reader,
            endian: endian,
            next: next,
        };

        Ok(x)
    }
    
    #[inline]
    fn read_ifd(&mut self) -> Result<IFD>  {
        self.reader.goto(self.next as u64)?;

        let mut ifd = IFD::new();
        for _ in 0..self.reader.read_u16(&self.endian)? {
            let (tag, entry) = self.read_entry()?;
            ifd.insert(tag, entry);
        }

        // Update next addr
        self.next = self.reader.read_u32(&self.endian)?;

        Ok(ifd)
    }

    #[inline]
    fn read_entry(&mut self) -> Result<(ifd::Tag, ifd::Entry)> {
        let tag = ifd::Tag::from_u16(self.reader.read_u16(&self.endian)?);
        let datatype = ifd::DataType::from_u16(self.reader.read_u16(&self.endian)?);
        
        let entry = Entry::new(
            datatype,
            self.reader.read_u32(&self.endian)?,
            self.reader.read_4byte()?,
        );

        Ok((tag, entry))
    }
}

impl<'a, R> Iterator for IFDs<'a, R> where R: Read + Seek + 'a {
    type Item = IFD;

    fn next(&mut self) -> Option<IFD> {
        if self.next == 0 {
            None
        } else {
            let ifd = self.read_ifd();
            let next = self.reader.read_u32(&self.endian);

            self.next = match next {
                Ok(next) => next,
                Err(_) => 0,
            };

            ifd.ok()
        }
    }
}

