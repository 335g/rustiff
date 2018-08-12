
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use error::{
    DecodeError,
    DecodeErrorKind,
    DecodeResult,
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
    DataType,
};
use tag::{
    TagKind,
};

use std::{
    io::{
        self,
        Read,
        Seek,
    },
};

use image::{
    BitsPerSample,
    Image,
    ImageHeader,
    Compression,
    PhotometricInterpretation,
};

#[derive(Debug)]
pub struct Decoder<R> {
    reader: R,
    endian: Endian,
    start: u32,
    next: u32,
}

impl<R> Decoder<R> where R: Read + Seek {
    pub fn new(mut reader: R) -> DecodeResult<Decoder<R>> {
        let mut byte_order = [0u8; 2];
        if let Err(e) = reader.read_exact(&mut byte_order) {
            return Err(DecodeError::from(DecodeErrorKind::NoByteOrder));
        }

        let endian = match &byte_order {
            b"II" => Endian::Little,
            b"MM" => Endian::Big,
            _ => return Err(DecodeError::from(DecodeErrorKind::NoByteOrder)),
        };

        match reader.read_u16(&endian) {
            Ok(x) if x == 42 => {},
            _ => return Err(DecodeError::from(DecodeErrorKind::NoVersion))
        }
        
        let start = match reader.read_u32(&endian) {
            Ok(x) => x,
            Err(_) => return Err(DecodeError::from(DecodeErrorKind::NoIFDAddress))
        };

        let decoder = Decoder {
            start: start,
            next: start,
            reader: reader,
            endian: endian,
        };

        Ok(decoder)
    }

    pub fn ifds(&mut self) -> Vec<IFD> {
        self.collect::<Vec<_>>()
    }

    #[inline]
    pub fn ifd(&mut self) -> DecodeResult<IFD> {
        let start = self.start;
        let (ifd, _) = self.read_ifd(start)?;
        Ok(ifd)
    }

    #[inline]
    pub fn get_entry<'a>(&mut self, ifd: &'a IFD, tag: &TagKind) -> DecodeResult<&'a Entry> {
        ifd.get(tag)
            .ok_or(DecodeError::from(DecodeErrorKind::CannotFindTheTag{ tag: tag.clone() }))
    }
    
    #[inline]
    fn going_to_get_it(&mut self, mut offset: &[u8], n: u32) -> DecodeResult<Vec<u32>> {
        self.reader.goto(offset.read_u32(&self.endian)? as u64)?;
        let mut data = Vec::with_capacity(n as usize);
        for _ in 0..n {
            data.push(self.reader.read_u16(&self.endian)? as u32);
        }

        Ok(data)
    }
    
    #[inline]
    pub fn get_entry_values(&mut self, ifd: &IFD, tag: &TagKind) -> DecodeResult<Vec<u32>> {
        let entry = self.get_entry(ifd, tag)?;

        let mut offset = entry.offset();

        match (entry.datatype(), entry.count()) {
            (DataType::Byte, 1) => Ok(vec![offset.read_u8()? as u32]),
            (DataType::Short, 1) => Ok(vec![offset.read_u16(&self.endian)? as u32]),
            (DataType::Short, 2) => {
                Ok(vec![
                    offset.read_u16(&self.endian)? as u32,
                    offset.read_u16(&self.endian)? as u32
                ])
            }
            (DataType::Short, n) if n >= 3 => self.going_to_get_it(&mut offset, n),
            (DataType::Long, 1) => Ok(vec![offset.read_u32(&self.endian)? as u32]),
            (DataType::Long, n) if n >= 2 => self.going_to_get_it(&mut offset, n),
            (DataType::Rational, n) => self.going_to_get_it(&mut offset, n),
            _ => {
                Err(DecodeError::from(
                    DecodeErrorKind::UnsupportedIFDEntry { 
                        entry: entry.clone(),
                        reason: "A suitable `DataType` & entry.count does not exist.".to_string(),
                    }))
            },
        }
    }

    #[inline]
    fn get_entry_values_u8(&mut self, ifd: &IFD, tag: &TagKind) -> DecodeResult<Vec<u8>> {
        let values = self.get_entry_values(ifd, tag)?;
        let mut v = vec![];
        for val in values {
            if val > u8::max_value() as u32 {
                return Err(DecodeError::from(DecodeErrorKind::OverflowValue{ tag: tag.clone(), value: val }))
            } else {
                v.push(val as u8);
            }
        }
        Ok(v)
    }
    
    #[inline]
    pub fn get_entry_value(&mut self, ifd: &IFD, tag: &TagKind) -> DecodeResult<u32> {
        let values = self.get_entry_values(ifd, tag)?;

        if values.len() > 1 {
            Err(DecodeError::from(
                DecodeErrorKind::UnsupportedIFDEntry { 
                    entry: self.get_entry(&ifd, tag)?.clone(),
                    reason: format!("tag({:?}) is need to some values.", tag),
                }))
        } else {
            // It should not come here, because `get_values` will not return empty `Ok(vec)`.
            Ok(*values.first().unwrap())
        }
    }

    #[inline]
    pub fn get_entry_value_u8(&mut self, ifd: &IFD, tag: &TagKind) -> DecodeResult<u8> {
        let value = self.get_entry_value(ifd, tag)?;
        if value > u8::max_value() as u32 {
            Err(DecodeError::from(DecodeErrorKind::OverflowValue { tag: tag.clone(), value: value }))
        } else {
            Ok(value as u8)
        }
    }
    
    #[inline]
    fn read_ifd(&mut self, from: u32) -> DecodeResult<(IFD, u32)>  {
        self.reader.goto(from as u64)?;

        let mut ifd = IFD::new(from);
        for _ in 0..self.reader.read_u16(&self.endian)? {
            let (tag, entry) = self.read_entry()?;
            ifd.insert(tag, entry);
        }

        let next = self.reader.read_u32(&self.endian)?;

        Ok((ifd, next))
    }
    
    #[inline]
    fn read_entry(&mut self) -> DecodeResult<(TagKind, Entry)> {
        let tag = TagKind::from_u16(self.reader.read_u16(&self.endian)?);
        let datatype = DataType::from_u16(self.reader.read_u16(&self.endian)?);
        
        let entry = Entry::new(
            datatype,
            self.reader.read_u32(&self.endian)?,
            self.reader.read_4byte()?,
        );

        Ok((tag, entry))
    }

    #[inline]
    pub fn header_from(&mut self, ifd: &IFD) -> DecodeResult<ImageHeader> {
        let width = self.get_entry_value(ifd, &TagKind::ImageWidth)?;
        let height = self.get_entry_value(ifd, &TagKind::ImageLength)?;
        let samples = self.get_entry_value(ifd, &TagKind::SamplesPerPixel).unwrap_or(1);
        let compression = self.get_entry_value(ifd, &TagKind::Compression).unwrap_or(1);
        let compression = Compression::from_u16(compression as u16)?;
        let interpretation = self.get_entry_value(ifd, &TagKind::PhotometricInterpretation)?;
        let interpretation = PhotometricInterpretation::from_u16(interpretation as u16)?;
        
        let bits = self.get_entry_values_u8(ifd, &TagKind::BitsPerSample).unwrap_or(vec![1]);
        let bits_len = bits.len();
        let bits_per_sample = if samples == bits_len as u32 {
            BitsPerSample::new(bits)
        } else {
            let err = DecodeErrorKind::NoMatchNumberOfSamples { 
                samples: samples as u8, 
                bits_per_sample: bits.into_iter().map(|x| x as u8).collect::<Vec<u8>>() 
            };
            
            return Err(DecodeError::from(err));
        };

        let header = ImageHeader::new(width, height, compression, interpretation, bits_per_sample);
        Ok(header)
    }
    
    #[inline]
    pub fn header(&mut self) -> DecodeResult<ImageHeader> {
        let ifd = self.ifd()?;
        self.header_from(&ifd)
    }

    #[inline]
    pub fn image_from(&mut self, ifd: &IFD) -> DecodeResult<Image> {
        let header = self.header_from(ifd)?;
        let width = header.width();
        let height = header.height();

        let bits_per_pixel = header.bits_per_sample()
            .into_iter()
            .map(|x| *x as u32)
            .sum::<u32>();
        let scanline_size = (width * bits_per_pixel + 7)/8;
        let rows_per_strip = self.get_entry_value(ifd, &TagKind::RowsPerStrip).unwrap_or(height);

        let offsets = self.get_entry_values(ifd, &TagKind::StripOffsets)?;
        let strip_byte_counts = self.get_entry_values(ifd, &TagKind::StripByteCounts)?;
        
        for (i, (&offset, &byte_count)) in offsets.iter()
            .zip(strip_byte_counts.iter())
            .enumerate()
        {
            let uncompressed_strip_size = scanline_size * (height - i as u32 * rows_per_strip);
            
        }

        unimplemented!()
    }
    
    pub fn image(&mut self) -> DecodeResult<Image> {
        let ifd = self.ifd()?;
        self.image_from(&ifd)
    }
}

impl<R> Iterator for Decoder<R> where R: Read + Seek {
    type Item = IFD;

    fn next(&mut self) -> Option<IFD> {
        let next = self.next;
        if let Some((ifd, next)) = self.read_ifd(next).ok() {
            self.next = next;

            Some(ifd)
        } else {
            None
        }
    }
}
