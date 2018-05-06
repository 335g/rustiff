
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use error::Result;
use byte::{
    Endian,
    EndianReadExt,
    ReadExt,
    SeekExt,
};
use ifd::{
    self,
    IFD,
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

    //pub ifds(&self) -> IFDs<R> {
    //    IFDs::new(self.reader, self.endian)
    //}
}

pub struct IFDs<R> {
    reader: R,
    endian: Endian,
    next: u32,
}

impl<R> IFDs<R> where R: Read + Seek {
    pub fn new(reader: R, endian: Endian) -> IFDs<R> {
        IFDs {
            reader: reader,
            endian: endian,
            next: 4, // initial IFD address
        }
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
        unimplemented!()
    }

}

impl<R> Iterator for IFDs<R> where R: Read + Seek {
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


