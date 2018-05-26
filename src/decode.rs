
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
    Tag,
    Entry,
    DataType,
};

use std::{
    io::{
        self,
        Read,
        Seek,
    },
};

use image::{
    Image,
};

#[derive(Debug)]
pub struct Decoder<R> {
    reader: R,
    endian: Endian,
    start: u32,
    next: u32,
}

impl<R> Decoder<R> where R: Read + Seek {
    pub fn new(mut reader: R) -> Result<Decoder<R>> {
        let mut byte_order = [0u8; 2];
        reader.read_exact(&mut byte_order)?;

        let endian = match &byte_order {
            b"II" => Ok(Endian::Little),
            b"MM" => Ok(Endian::Big),
            _ => Err(Error::from(DecodeError::IncorrectHeader{ reason: "Not byteorder".to_string() })),
        }?;

        if reader.read_u16(&endian)? != 42 {
            return Err(Error::from(DecodeError::IncorrectHeader{ reason: "Not 42".to_string() }));
        }

        let start = reader.read_u32(&endian)?;

        let decoder = Decoder {
            start: start,
            next: start,
            reader: reader,
            endian: endian,
        };

        Ok(decoder)
    }

    pub fn load_ifds(&mut self) {
        self.next = self.start;
    }

    pub fn ifd(&mut self) -> Option<IFD> {
        self.into_iter().next()
    }

    pub fn get_entry<'a>(&mut self, ifd: &'a IFD, tag: &Tag) -> Result<&'a Entry> {
        let entry = ifd.get(tag)
            .ok_or(Error::from(DecodeError::CannotFindTheTag{ tag: tag.clone() }))?;
        Ok(entry)
    }
    
    #[inline]
    fn going_to_get_it(&mut self, mut offset: &[u8], n: u32) -> Result<Vec<u32>> {
        self.reader.goto(offset.read_u32(&self.endian)? as u64)?;
        let mut data = Vec::with_capacity(n as usize);
        for _ in 0..n {
            data.push(self.reader.read_u16(&self.endian)? as u32);
        }

        Ok(data)
    }
    
    #[inline]
    pub fn get_entry_values(&mut self, ifd: &IFD, tag: &Tag) -> Result<Vec<u32>> {
        let entry = self.get_entry(&ifd, &tag)?;

        let mut offset = entry.offset();

        match (entry.datatype(), entry.count()) {
            (&DataType::Byte, 1) => Ok(vec![offset.read_u8()? as u32]),
            (&DataType::Short, 1) => Ok(vec![offset.read_u16(&self.endian)? as u32]),
            (&DataType::Short, 2) => {
                Ok(vec![
                    offset.read_u16(&self.endian)? as u32,
                    offset.read_u16(&self.endian)? as u32
                ])
            }
            (&DataType::Short, n) if n >= 3 => self.going_to_get_it(&mut offset, n),
            (&DataType::Long, 1) => Ok(vec![offset.read_u32(&self.endian)? as u32]),
            (&DataType::Long, n) if n >= 2 => self.going_to_get_it(&mut offset, n),
            (&DataType::Rational, n) => self.going_to_get_it(&mut offset, n),
            (dt, _) => Err(Error::from(DecodeError::UnsupportedDataType { datatype: dt.clone() })),
        }
    }
    
    #[inline]
    pub fn get_entry_value(&mut self, ifd: &IFD, tag: &Tag) -> Result<u32> {
        let values = self.get_entry_values(&ifd, &tag)?;

        if values.len() > 1 {
            Err(Error::from(DecodeError::ALot{ tag: tag.clone() }))
        } else if let Some(value) = values.first() {
            Ok(*value)
        } else {
            // It should not come here, because `get_values` will not return empty `Ok(vec)`.
            Err(Error::from(DecodeError::Few{ tag: tag.clone() }))
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
    fn read_entry(&mut self) -> Result<(Tag, Entry)> {
        let tag = Tag::from_u16(self.reader.read_u16(&self.endian)?);
        let datatype = DataType::from_u16(self.reader.read_u16(&self.endian)?);
        
        let entry = Entry::new(
            datatype,
            self.reader.read_u32(&self.endian)?,
            self.reader.read_4byte()?,
        );

        Ok((tag, entry))
    }

    pub fn image(&mut self, ifd: &IFD) -> Result<Image> {
        let width = self.get_entry_value(ifd, &Tag::ImageWidth)?;
        let height = self.get_entry_value(ifd, &Tag::ImageLength)?;

        unimplemented!()
    }

}

impl<R> Iterator for Decoder<R> where R: Read + Seek {
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

