
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
    LZWReader,
};
use ifd::{
    self,
    IFD,
    Entry,
    DataType,
};
use tag::{
    self,
    TagType,
    AnyTag,
};
use std::io::{
    self,
    Read,
    Seek,
};
use image::{
    BitsPerSample,
    Image,
    ImageData,
    ImageHeader,
    Compression,
    PhotometricInterpretation,
};
use failure::Fail;

macro_rules! read_byte {
    ($method:ident, $method2:ident, $typestr:ident, $t:ty) => {
        #[inline]
        fn $method(&mut self, ifd: &IFD, header: &ImageHeader, buffer_size: usize) -> DecodeResult<ImageData> {
            let interpretation = header.photometric_interpretation();
            let compression = header.compression();
            let width = header.width();
            let height = header.height();
            let bits_per_sample = header.bits_per_sample();
            let bits_per_pixel: usize = match bits_per_sample {
                BitsPerSample::U8_1 => 8,
                BitsPerSample::U8_3 => 24,
                BitsPerSample::U8_4 => 32,
                BitsPerSample::U16_1 => 16,
                BitsPerSample::U16_3 => 48,
                BitsPerSample::U16_4 => 64,
            };
            let scanline_size = (width as usize * bits_per_pixel + 7) / 8;

            let offsets = self.get_value(ifd, tag::StripOffsets)?;
            let strip_byte_counts = self.get_value(ifd, tag::StripByteCounts)?;
            let rows_per_strip = self.get_value(ifd, tag::RowsPerStrip)?; // TODO: default value is OK?
            let endian = self.endian;

            let mut buffer: Vec<$t> = vec![0; buffer_size];
            let mut read_size = 0;
            for (i, (offset, byte_count)) in offsets.into_iter().zip(strip_byte_counts.into_iter()).enumerate() {
                let offset = offset as usize;
                let byte_count = byte_count as usize;
                let uncompressed_size = scanline_size * (height as usize - i * rows_per_strip as usize);

                self.reader.goto(offset as u64)?;

                read_size += match compression {
                    Compression::No => $method2(
                        interpretation,
                        read_size,
                        buffer_size,
                        endian,
                        (&mut self.reader, byte_count),
                        &mut buffer[read_size..])?,

                    Compression::LZW => $method2(
                        interpretation,
                        read_size,
                        buffer_size,
                        endian,
                        LZWReader::new(&mut self.reader, byte_count, uncompressed_size)?,
                        &mut buffer[read_size..])?,
                };
            }
            buffer.shrink_to_fit();

            Ok(ImageData::$typestr(buffer))
        }
    }
}

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

        match reader.read_u16(endian) {
            Ok(x) if x == 42 => {},
            _ => return Err(DecodeError::from(DecodeErrorKind::NoVersion))
        }
        let start = match reader.read_u32(endian) {
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

    pub fn ifd(&mut self) -> DecodeResult<IFD> {
        let start = self.start;
        let (ifd, _) = self.read_ifd(start)?;
        Ok(ifd)
    }

    fn get_entry<'a, T: TagType>(&mut self, ifd: &'a IFD, tag: T) -> DecodeResult<&'a Entry> {
        ifd.get(tag).ok_or(DecodeError::from(DecodeErrorKind::CannotFindTheTag{ tag: AnyTag::from(tag) }))
    }
    
    fn going_to_get_it(&mut self, mut offset: &[u8], n: u32) -> DecodeResult<Vec<u32>> {
        self.reader.goto(offset.read_u32(self.endian)? as u64)?;
        let mut data = Vec::with_capacity(n as usize);
        for _ in 0..n {
            data.push(self.reader.read_u16(self.endian)? as u32);
        }

        Ok(data)
    }

    pub fn get_value<T: TagType>(&mut self, ifd: &IFD, tag: T) -> DecodeResult<T::Value> {
        match T::default_value() {
            None => tag.value_from(self.get_entry_u32_values(ifd, tag)?),
            Some(def) => tag.value_from(self.get_entry_u32_values(ifd, tag)?)
                .or_else(|e| {
                    match e.kind() {
                        DecodeErrorKind::CannotFindTheTag { tag: _ } => Ok(def),
                        _ => Err(e),
                    }
                }),
        }
    }

    fn get_entry_u32_values<T: TagType>(&mut self, ifd: &IFD, tag: T) -> DecodeResult<Vec<u32>> {
        let entry = self.get_entry(ifd, tag)?;

        let mut offset = entry.offset();

        match (entry.datatype(), entry.count()) {
            (DataType::Byte, 1) => Ok(vec![offset.read_u8()? as u32]),
            (DataType::Short, 1) => Ok(vec![offset.read_u16(self.endian)? as u32]),
            (DataType::Short, 2) => {
                Ok(vec![
                    offset.read_u16(self.endian)? as u32,
                    offset.read_u16(self.endian)? as u32
                ])
            }
            (DataType::Short, n) if n >= 3 => self.going_to_get_it(&mut offset, n),
            (DataType::Long, 1) => Ok(vec![offset.read_u32(self.endian)? as u32]),
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

    fn read_ifd(&mut self, from: u32) -> DecodeResult<(IFD, u32)>  {
        self.reader.goto(from as u64)?;

        let mut ifd = IFD::new();
        for _ in 0..self.reader.read_u16(self.endian)? {
            let (tag, entry) = self.read_entry()?;
            ifd.insert_anytag(tag, entry);
        }

        let next = self.reader.read_u32(self.endian)?;

        Ok((ifd, next))
    }
    
    fn read_entry(&mut self) -> DecodeResult<(AnyTag, Entry)> {
        let tag = AnyTag::from_u16(self.reader.read_u16(self.endian)?);
        let datatype = DataType::from_u16(self.reader.read_u16(self.endian)?);
        
        let entry = Entry::new(
            datatype,
            self.reader.read_u32(self.endian)?,
            self.reader.read_4byte()?,
        );

        Ok((tag, entry))
    }

    pub fn header_with(&mut self, ifd: &IFD) -> DecodeResult<ImageHeader> {
        let width = self.get_value(ifd, tag::ImageWidth)?;
        let height = self.get_value(ifd, tag::ImageLength)?;
        let compression = self.get_value(ifd, tag::Compression)?;
        let compression = Compression::from_u16(compression)?;
        let interpretation = self.get_value(ifd, tag::PhotometricInterpretation)?;
        let interpretation = PhotometricInterpretation::from_u16(interpretation)?;
        let bits = self.get_value(ifd, tag::BitsPerSample)?;
        let bits_per_sample = BitsPerSample::new(bits)?;
        let header = ImageHeader::new(width, height, compression, interpretation, bits_per_sample)?;
        
        Ok(header)
    }
    
    pub fn header(&mut self) -> DecodeResult<ImageHeader> {
        let ifd = self.ifd()?;
        self.header_with(&ifd)
    }
    
    read_byte!(read_byte_u8, read_byte_detail_u8, U8, u8);
    read_byte!(read_byte_u16, read_byte_detail_u16, U16, u16);

    pub fn image_with(&mut self, ifd: &IFD) -> DecodeResult<Image> {
        let header = self.header_with(ifd)?;
        let width = header.width() as usize;
        let height = header.height() as usize;
        let interpretation = header.photometric_interpretation();
        let bits_per_sample = header.bits_per_sample();
        let compression = header.compression();
        let buffer_size = width * height * header.bits_per_sample().len();
        let endian = self.endian;
        let data = match bits_per_sample {
            BitsPerSample::U8_1 | BitsPerSample::U8_3 | BitsPerSample::U8_4 => self.read_byte_u8(ifd, &header, buffer_size),
            BitsPerSample::U16_1 | BitsPerSample::U16_3 | BitsPerSample::U16_4 => self.read_byte_u16(ifd, &header, buffer_size),
        }?;
        
        Ok(Image::new(header, data))
    }
    
    pub fn image(&mut self) -> DecodeResult<Image> {
        let ifd = self.ifd()?;
        self.image_with(&ifd)
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

fn read_byte_detail_u16<S>(
    interpretation: PhotometricInterpretation,
    read_size: usize,
    buffer_size: usize,
    endian: Endian,
    reader_and_size: (S, usize),
    buffer: &mut [u16]) -> DecodeResult<usize> where S: Read
{
    let mut reader = reader_and_size.0;
    let compressed_size = reader_and_size.1;

    if read_size + compressed_size > buffer_size {
        return Err(DecodeError::from(DecodeErrorKind::IncorrectBufferSize { calc: buffer_size, sum: read_size + compressed_size }));
    }
    
    for data in buffer[..compressed_size/2].iter_mut() {
        *data = if interpretation == PhotometricInterpretation::BlackIsZero {
            u16::max_value() - reader.read_u16(endian)?
        } else {
            reader.read_u16(endian)?
        };
    }

    Ok(compressed_size/2)
}

fn read_byte_detail_u8<S>(
    interpretation: PhotometricInterpretation, 
    read_size: usize,
    buffer_size: usize,
    _endian: Endian,
    reader_and_size: (S, usize),
    buffer: &mut [u8]) -> DecodeResult<usize> where S: Read,
{
    let mut reader = reader_and_size.0;
    let compressed_size = reader_and_size.1;
    if read_size + compressed_size > buffer_size {
        return Err(DecodeError::from(DecodeErrorKind::IncorrectBufferSize { calc: buffer_size, sum: read_size + compressed_size }));
    }

    let res = reader.read(&mut buffer[..compressed_size])?;
    if interpretation == PhotometricInterpretation::BlackIsZero {
        for data in buffer[..compressed_size].iter_mut() {
            *data = u8::max_value() - *data;
        }
    }
    Ok(res)
}

