
use error::{
    DecodeError,
    DecodeErrorKind,
    FileHeaderErrorKind,
};
use byte::{
    Endian,
    ReadExt,
    SeekExt,
    LZWReader,
};
use ifd::{
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
    Read,
    Seek,
};
use image::{
    BitsPerSample,
    Image,
    ImageData,
    ImageHeaderBuilder,
    ImageHeader,
    Compression,
    PhotometricInterpretation,
    ConstructError,
};

macro_rules! read_byte {
    ($method:ident, $method2:ident, $t:ty) => {
        fn $method(&mut self, ifd: &IFD, header: &ImageHeader) -> Result<Vec<$t>, DecodeError> {
            let bits_per_sample = header.bits_per_sample().bits();
            let width = header.width();
            let height = header.height();
            let samples_per_pixel = header.samples_per_pixel();
            let buffer_size = (width * height * samples_per_pixel as u32) as usize;
            let interpretation = header.photometric_interpretation();
            let compression = header.compression();
            let offsets = self.get_value(ifd, tag::StripOffsets)?;
            let strip_byte_counts = self.get_value(ifd, tag::StripByteCounts)?;
            
            let mut buffer: Vec<$t> = vec![0; buffer_size];
            let mut read_size = 0;
            for (offset, byte_count) in offsets.into_iter().zip(strip_byte_counts.into_iter()) {
                let byte_count = byte_count as usize;
                
                self.reader.goto(u64::from(offset))?;

                read_size += match compression {
                    None => {
                        let will_read_size = read_size + byte_count;
                        if will_read_size > buffer_size {
                            return Err(DecodeError::from(DecodeErrorKind::IncorrectBufferSize { want: buffer_size, got: will_read_size }))
                        }

                        $method2(interpretation, self.endian, byte_count, &mut self.reader, &mut buffer[read_size..])?;
                        byte_count
                    }

                    Some(Compression::LZW) => {
                        let (mut reader, uncompressed_size) = LZWReader::new(&mut self.reader, byte_count)?;
                        let will_read_size = read_size + uncompressed_size;
                        if will_read_size > buffer_size {
                            return Err(DecodeError::from(DecodeErrorKind::IncorrectBufferSize { want: buffer_size, got: will_read_size }))
                        }
                        
                        $method2(interpretation, self.endian, uncompressed_size, &mut reader, &mut buffer[read_size..])?;
                        uncompressed_size
                    }
                };
            }

            if read_size != buffer_size {
                Err(DecodeError::from(DecodeErrorKind::IncorrectBufferSize { want: buffer_size, got: read_size }))
            } else {
                Ok(buffer)
            }
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
    pub fn new(mut reader: R) -> Result<Decoder<R>, DecodeError> {
        let mut byte_order = [0u8; 2];
        if let Err(_) = reader.read_exact(&mut byte_order) {
            return Err(DecodeError::from(FileHeaderErrorKind::NoByteOrder));
        }
        let endian = match &byte_order {
            b"II" => Endian::Little,
            b"MM" => Endian::Big,
            _ => return Err(DecodeError::from(FileHeaderErrorKind::NoByteOrder)),
        };

        match reader.read_u16(endian) {
            Ok(x) if x == 42 => {},
            _ => return Err(DecodeError::from(FileHeaderErrorKind::NoVersion))
        }
        let start = match reader.read_u32(endian) {
            Ok(x) => x,
            Err(_) => return Err(DecodeError::from(FileHeaderErrorKind::NoIFDAddress))
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

    pub fn ifd(&mut self) -> Result<IFD, DecodeError> {
        let start = self.start;
        let (ifd, _) = self.read_ifd(start)?;
        Ok(ifd)
    }

    fn get_entry<'a, T: TagType>(&mut self, ifd: &'a IFD, tag: T) -> Result<&'a Entry, DecodeError> {
        ifd.get(tag)?
            .ok_or(DecodeError::from(DecodeErrorKind::CannotFindTheTag{ tag: Box::new(tag) }))
    }
    
    pub fn get_value<T: TagType>(&mut self, ifd: &IFD, tag: T) -> Result<T::Value, DecodeError> {
        let entry = self.get_entry(ifd, tag)?;
        tag.decode(&mut self.reader, entry.offset(), self.endian, entry.datatype(), entry.count() as usize)
    }

    fn read_ifd(&mut self, from: u32) -> Result<(IFD, u32), DecodeError>  {
        self.reader.goto(from as u64)?;

        let mut ifd = IFD::new();
        for _ in 0..self.reader.read_u16(self.endian)? {
            let tag = AnyTag::from_u16(self.reader.read_u16(self.endian)?);
            let datatype = DataType::from(self.reader.read_u16(self.endian)?);
            let count = self.reader.read_u32(self.endian)?;
            let offset = self.reader.read_4byte()?;
            let entry = Entry::new(datatype, count, offset);
            ifd.insert_anytag(tag, entry);
        }

        let next = self.reader.read_u32(self.endian)?;

        Ok((ifd, next))
    }
    
    #[inline]
    pub fn header_with(&mut self, ifd: &IFD) -> Result<ImageHeader, DecodeError> {
        let width = self.get_value(ifd, tag::ImageWidth)?;
        let height = self.get_value(ifd, tag::ImageLength)?;
        let interpretation = PhotometricInterpretation::from_u16(self.get_value(ifd, tag::PhotometricInterpretation)?)?;
        let bits_per_sample = BitsPerSample::from_u16s(self.get_value(ifd, tag::BitsPerSample)?)?;
        let samples_per_pixel = self.get_value(ifd, tag::SamplesPerPixel)?;
        let builder = ImageHeaderBuilder::default()
            .width(width)
            .height(height)
            .photometric_interpretation(interpretation)
            .bits_per_sample(bits_per_sample)
            .samples_per_pixel(samples_per_pixel);

        let compression = self.get_value(ifd, tag::Compression)?;
        let builder = if let Some(compression) = Compression::from_u16(compression)? {
            builder.compression(compression)
        } else {
            builder
        };

        let header = builder.build()?;

        Ok(header)
    }
    
    pub fn header(&mut self) -> Result<ImageHeader, DecodeError> {
        let ifd = self.ifd()?;

        self.header_with(&ifd)
    }
    
    read_byte!(read_byte_only_u8, read_u8s, u8);
    read_byte!(read_byte_only_u16, read_u16s, u16);

    pub fn image(&mut self) -> Result<Image, DecodeError> {
        let ifd = self.ifd()?;
        self.image_with(&ifd)
    }
    
    #[inline]
    pub fn image_with(&mut self, ifd: &IFD) -> Result<Image, DecodeError> {
        let header = self.header_with(ifd)?;
        let bits_per_sample = header.bits_per_sample().bits().clone();
        let data = if bits_per_sample.iter().all(|&n| n <= 8) {
            ImageData::U8(self.read_byte_only_u8(ifd, &header)?)

        } else if bits_per_sample.iter().all(|&n| 8 < n && n <= 16) {
            ImageData::U16(self.read_byte_only_u16(ifd, &header)?)

        } else if bits_per_sample.iter().all(|&n| n <= 16) {
            ImageData::U16(self.read_byte_u8_or_u16(ifd, &header)?)

        } else {
            return Err(DecodeError::from(ConstructError::new(
                tag::BitsPerSample,
                bits_per_sample,
                "tag::BitsPerSample must have a value less than 16.".to_string()
            )));
        };
        
        Ok(Image::new(header, data))
    }

    fn read_byte_u8_or_u16(&mut self, ifd: &IFD, header: &ImageHeader) -> Result<Vec<u16>, DecodeError> {
        let bits_per_sample = header.bits_per_sample().bits();
        let width = header.width();
        let height = header.height();
        let samples_per_pixel = header.samples_per_pixel();
        let buffer_size = (width * height * samples_per_pixel as u32) as usize;
        let interpretation = header.photometric_interpretation();
        let compression = header.compression();
        let offsets = self.get_value(ifd, tag::StripOffsets)?;
        let strip_byte_counts = self.get_value(ifd, tag::StripByteCounts)?;
        let endian = self.endian;

        let mut buffer: Vec<u16> = vec![0; buffer_size];
        let mut read_size = 0;
        for (offset, byte_count) in offsets.into_iter().zip(strip_byte_counts.into_iter()) {
            let byte_count = byte_count as usize;

            self.reader.goto(u64::from(offset))?;

            read_size += match compression {
                None => {
                    let will_read_size = read_size + byte_count;
                    if will_read_size > buffer_size {
                        return Err(DecodeError::from(DecodeErrorKind::IncorrectBufferSize { want: buffer_size, got: will_read_size }))
                    }

                    let mut read_size_now = 0;
                    
                    while read_size_now >= byte_count {
                        for bits in bits_per_sample.iter() {
                            if *bits <= 8 {
                                let data = read_u8(interpretation, &mut self.reader)?;
                                read_size_now += 1;
                                buffer.push(u16::from(data));

                            } else if *bits <= 16 {
                                let data = read_u16(interpretation, endian, &mut self.reader)?;
                                read_size_now += 2;
                                buffer.push(data);

                            } else {
                                return Err(DecodeError::from(ConstructError::new(
                                    tag::BitsPerSample,
                                    bits_per_sample.clone(),
                                    "tag::BitsPerSample must have a value less than 16.".to_string()
                                )));
                            }
                        }
                    }
                    // TODO: return error when read_size_now > byte_count
                    
                    read_size_now
                }
                
                Some(Compression::LZW) => {
                    let (mut reader, uncompressed_size) = LZWReader::new(&mut self.reader, byte_count)?;
                    let will_read_size = read_size + uncompressed_size;
                    if will_read_size > buffer_size {
                        return Err(DecodeError::from(DecodeErrorKind::IncorrectBufferSize { want: buffer_size, got: will_read_size }))
                    }
                    
                    let mut read_size_now = 0;
                    
                    while read_size_now >= byte_count {
                        for bits in bits_per_sample.iter() {
                            if *bits <= 8 {
                                let data = read_u8(interpretation, &mut reader)?;
                                read_size_now += 1;
                                buffer.push(u16::from(data));

                            } else if *bits <= 16 {
                                let data = read_u16(interpretation, endian, &mut reader)?;
                                read_size_now += 2;
                                buffer.push(data);

                            } else {
                                return Err(DecodeError::from(ConstructError::new(
                                    tag::BitsPerSample,
                                    bits_per_sample.clone(),
                                    "tag::BitsPerSample must have a value less than 16.".to_string()
                                )));
                            }
                        }
                    }
                    // TODO: return error when read_size_now > byte_count

                    read_size_now
                }
            }
        }

        if read_size != buffer_size {
            Err(DecodeError::from(DecodeErrorKind::IncorrectBufferSize { want: buffer_size, got: read_size }))
        } else {
            Ok(buffer)
        }
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

#[inline(always)]
fn read_u16s<R>(interpretation: PhotometricInterpretation, endian: Endian, length: usize, mut reader: R, buffer: &mut [u16]) -> Result<(), DecodeError> where R: Read {
    reader.read_u16_into(endian, &mut buffer[..length/2])?;
    if interpretation == PhotometricInterpretation::BlackIsZero {
        for data in buffer[..length/2].iter_mut() {
            *data = u16::max_value() - *data;
        }
    }
    Ok(())
}

#[inline(always)]
fn read_u16<R>(interpretation: PhotometricInterpretation, endian: Endian, mut reader: R) -> Result<u16, DecodeError> where R: Read {
    let mut value = reader.read_u16(endian)?;
    if interpretation == PhotometricInterpretation::BlackIsZero {
        value = u16::max_value() - value;
    }
    Ok(value)
}

#[inline(always)]
fn read_u8s<R>(interpretation: PhotometricInterpretation, _endian: Endian, length: usize, mut reader: R, buffer: &mut [u8]) -> Result<(), DecodeError> where R: Read {
    reader.read_exact(&mut buffer[..length])?;
    if interpretation == PhotometricInterpretation::BlackIsZero {
        for data in buffer[..length].iter_mut() {
            *data = u8::max_value() - *data;
        }
    }
    Ok(())
}

#[inline(always)]
fn read_u8<R>(interpretation: PhotometricInterpretation, mut reader: R) -> Result<u8, DecodeError> where R: Read {
    let mut value = reader.read_u8()?;
    if interpretation == PhotometricInterpretation::BlackIsZero {
        value = u8::max_value() - value;
    }
    Ok(value)
}

