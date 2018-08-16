
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

use failure::Fail;

macro_rules! get_entry_values {
    ($method:ident, $overflow:ident, $t:ident) => {
        #[inline]
        pub fn $method(&mut self, ifd: &IFD, tag: &TagKind) -> DecodeResult<Vec<$t>> {
            let values = self.get_entry_u32_values(ifd, tag)?;
            let mut v = vec![];
            for val in values {
                if val > $t::max_value() as u32 {
                    return Err(DecodeError::from(DecodeErrorKind::$overflow { tag: tag.clone(), value: val }));
                } else {
                    v.push(val as $t);
                }
            }
            Ok(v)
        }
    }
}

macro_rules! get_entry_value {
    ($method:ident, $overflow:ident, $t:ident) => {
        #[inline]
        pub fn $method(&mut self, ifd: &IFD, tag: &TagKind) -> DecodeResult<$t> {
            let value = self.get_entry_u32_value(ifd, tag)?;
            if value > $t::max_value() as u32 {
                Err(DecodeError::from(DecodeErrorKind::$overflow { tag: tag.clone(), value: value }))
            } else {
                Ok(value as $t)
            }
        }
    }
}

macro_rules! set_default_values_when_there_is_no_key {
    ($method:ident, $method2:ident, $t:ty) => {
        #[inline]
        fn $method<T: AsRef<[$t]>>(&mut self, ifd: &IFD, tag: &TagKind, def: T) -> DecodeResult<Vec<$t>> {
            self.$method2(ifd, tag)
                .or_else(|e| {
                    match e.kind() {
                        DecodeErrorKind::CannotFindTheTag{ tag: _ } => Ok(def.as_ref().to_vec()),
                        _ => Err(e)
                    }
                })
        }
    }
}

macro_rules! set_default_value_when_there_is_no_key {
    ($method:ident, $method2:ident, $t:ty) => {
        #[inline]
        fn $method(&mut self, ifd: &IFD, tag: &TagKind, def: $t) -> DecodeResult<$t> {
            self.$method2(ifd, tag)
                .or_else(|e| {
                    match e.kind() {
                        DecodeErrorKind::CannotFindTheTag{ tag: _ } => Ok(def),
                        _ => Err(e)
                    }
                })
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
    pub fn get_entry_u32_values(&mut self, ifd: &IFD, tag: &TagKind) -> DecodeResult<Vec<u32>> {
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

    get_entry_values!(get_entry_u8_values, OverflowU8Value, u8);
    get_entry_values!(get_entry_u16_values, OverflowU16Value, u16);

    set_default_values_when_there_is_no_key!(get_entry_u32_values_or, get_entry_u32_values, u32);
    set_default_values_when_there_is_no_key!(get_entry_u16_values_or, get_entry_u16_values, u16);
    set_default_values_when_there_is_no_key!(get_entry_u8_values_or, get_entry_u8_values, u8);

    #[inline]
    pub fn get_entry_u32_value(&mut self, ifd: &IFD, tag: &TagKind) -> DecodeResult<u32> {
        let values = self.get_entry_u32_values(ifd, tag)?;

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

    get_entry_value!(get_entry_u8_value, OverflowU8Value, u8);
    get_entry_value!(get_entry_u16_value, OverflowU16Value, u16);

    set_default_value_when_there_is_no_key!(get_entry_u8_value_or, get_entry_u8_value, u8);
    set_default_value_when_there_is_no_key!(get_entry_u16_value_or, get_entry_u16_value, u16);
    set_default_value_when_there_is_no_key!(get_entry_u32_value_or, get_entry_u32_value, u32);
    
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
    pub fn header_with(&mut self, ifd: &IFD) -> DecodeResult<ImageHeader> {
        let width = self.get_entry_u32_value(ifd, &TagKind::ImageWidth)?;
        let height = self.get_entry_u32_value(ifd, &TagKind::ImageLength)?;
        let samples = self.get_entry_u32_value(ifd, &TagKind::SamplesPerPixel).unwrap_or(1);
        let compression = self.get_entry_u16_value(ifd, &TagKind::Compression).unwrap_or(1);
        let compression = Compression::from_u16(compression)?;
        let interpretation = self.get_entry_u16_value(ifd, &TagKind::PhotometricInterpretation)?;
        let interpretation = PhotometricInterpretation::from_u16(interpretation)?;
        
        let bits = self.get_entry_u8_values_or(ifd, &TagKind::BitsPerSample, vec![1])?;
        //let bits_len = bits.len();
        //let bits_per_sample = if samples == bits_len as u32 {
        //    BitsPerSample::new(bits)
        //} else {
        //    let err = DecodeErrorKind::IncorrectNumberOfSamples { 
        //        samples: samples as u8, 
        //        bits_per_sample: bits.into_iter().map(|x| x as u8).collect::<Vec<u8>>() 
        //    };
        //    
        //    return Err(DecodeError::from(err));
        //};
        
        //let bits_per_sample = if samples != bits

        //let header = ImageHeader::new(width, height, compression, interpretation, bits_per_sample);
        //Ok(header)

        unimplemented!()
    }
    
    #[inline]
    pub fn header(&mut self) -> DecodeResult<ImageHeader> {
        let ifd = self.ifd()?;
        self.header_with(&ifd)
    }

    #[inline]
    pub fn image_with(&mut self, ifd: &IFD) -> DecodeResult<Image> {
        let header = self.header_with(ifd)?;
        let width = header.width() as usize;
        let height = header.height() as usize;
        let buffer_size = width * height * header.bits_per_sample().len();
        let mut buffer = Vec::<u8>::with_capacity(buffer_size);
        let mut read: usize = 0;
        let mut read_now: usize = 0;

        let offsets = self.get_entry_u32_values(ifd, &TagKind::StripOffsets)?;
        let strip_byte_counts = self.get_entry_u32_values(ifd, &TagKind::StripByteCounts)?;
        let compression = header.compression();

        match compression {
            Compression::No => {
                for (i, (offset, byte_count)) in offsets.into_iter().zip(strip_byte_counts.into_iter()).enumerate() {
                    let offset = offset as usize;
                    let byte_count = byte_count as usize;

                    if read + byte_count > buffer_size {
                        return Err(DecodeError::from(DecodeErrorKind::IncorrectBufferSize { calc: buffer_size, sum: read + byte_count }));
                    }

                    //let reader = StrictReader::new(&mut self.reader, self.endian);
                    //let _ = self.read_byte(reader, &mut buffer, read, byte_count)?;
                    //read += byte_count;
                }
            }
            Compression::LZW => {
                //let bits_per_pixel = header.bits_per_sample().bits().iter().sum::<u8>();
                let bits_per_pixel = match header.bits_per_sample() {
                    BitsPerSample::U8_3 => 24,
                    BitsPerSample::U8_4 => 32,
                    BitsPerSample::U16_3 => 48,
                    BitsPerSample::U16_4 => 64,
                    BitsPerSample::N(n) => *n,
                };
                let rows_per_strip = self.get_entry_u32_value(ifd, &TagKind::RowsPerStrip).map(|x| x as usize).unwrap_or(height);
                let scanline_size_bits = bits_per_pixel as usize * width;
                let scanline_size = (scanline_size_bits + 7)/8;
                
                for (i, (offset, byte_count)) in offsets.into_iter().zip(strip_byte_counts.into_iter()).enumerate() {
                    let uncompressed_strip_size = scanline_size * (height - i * rows_per_strip);
                    
                }
                
                unimplemented!()
            }
        }
        
        Ok(Image::new(header, buffer))
    }
    
    pub fn image(&mut self) -> DecodeResult<Image> {
        let ifd = self.ifd()?;
        self.image_with(&ifd)
    }

    //fn read_byte<S: Read>(&mut self, ifd: &IFD, reader: S, buffer: &mut [u8], from: usize, size: usize) -> io::Result<usize> {
    //    
    //    unimplemented!()
    //}
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
