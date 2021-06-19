use io::SeekFrom;

use crate::{
    data::{AnyElements, DataType, Entry, Rational},
    element::{Elemental, Endian, EndianRead, SeekExt},
    error::{DecodeError, DecodingError, FileHeaderError, TagError},
    ifd::ImageFileDirectory,
    possible::Possible,
    tag::{self, AnyTag, Tag},
    val::Predictor,
    DecodeResult,
};
use std::{convert::TryFrom, io};

macro_rules! read {
    ($count:ident, $reader:ident, $func:ident, $anydata:path, $endian:ident) => {{
        let vals = (0..$count)
            .into_iter()
            .map(|_| $reader.$func(&$endian))
            .collect::<Result<Vec<_>, _>>()?;
        $anydata(vals)
    }};
}

pub trait Decoded: Sized {
    type Element: Elemental;
    type Poss: Possible;

    const POSSIBLE_COUNT: Self::Poss;

    fn decoded(elements: Vec<Self::Element>) -> Result<Self, DecodingError>;
}

#[derive(Debug)]
pub struct Decoder<R> {
    reader: R,
    endian: Endian,
    addr_index: usize,
    addrs: Vec<u64>,
    ifd: ImageFileDirectory,
}

impl<R> Decoder<R> {
    #[allow(missing_docs)]
    pub fn endian(&self) -> &Endian {
        &self.endian
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub(crate) fn reader_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    #[allow(missing_docs)]
    pub fn ifd(&self) -> &ImageFileDirectory {
        &self.ifd
    }

    #[allow(missing_docs)]
    pub fn get_entry<T: Tag>(&self) -> Result<Option<&Entry>, TagError<T>> {
        let ifd = self.ifd();

        self.get_entry_with::<T>(ifd)
    }

    #[allow(missing_docs)]
    fn get_entry_with<'a, 'b, T>(
        &'a self,
        ifd: &'b ImageFileDirectory,
    ) -> Result<Option<&'b Entry>, TagError<T>>
    where
        T: Tag,
        'a: 'b,
    {
        let anytag = AnyTag::try_from::<T>()?;

        let entry = ifd.get_tag(&anytag);
        Ok(entry)
    }

    #[allow(dead_code)]
    pub fn addresses<'a>(&'a mut self) -> Addresses<'a, R> {
        let start = *self.addrs.first().unwrap();
        let endian = self.endian().clone();
        let reader = self.reader_mut();

        Addresses::new(start, endian, reader)
    }
}

impl<R> Decoder<R>
where
    R: io::Read + io::Seek,
{
    pub fn new(mut reader: R) -> Result<Decoder<R>, DecodeError> {
        let mut byte_order = [0u8; 2];
        reader
            .read_exact(&mut byte_order)
            .map_err(|_| FileHeaderError::NoByteOrder)?;

        let endian = match &byte_order {
            b"II" => Endian::Little,
            b"MM" => Endian::Big,
            _ => {
                return Err(DecodeError::from(FileHeaderError::InvalidByteOrder {
                    byte_order: byte_order,
                }))
            }
        };

        let _ = reader
            .read_u16(&endian)
            .map_err(|_| FileHeaderError::NoVersion)
            .and_then(|n| {
                if n == 42 {
                    Ok(())
                } else {
                    Err(FileHeaderError::InvalidVersion { version: n })
                }
            })?;

        let start: u64 = reader
            .read_u32(&endian)
            .map_err(|_| FileHeaderError::NoIFDAddress)?
            .into();
        let ifd = Decoder::load_ifd_with(&mut reader, &endian, start)?;
        let decoder = Decoder {
            reader,
            endian,
            addr_index: 0,
            addrs: vec![start],
            ifd,
        };

        Ok(decoder)
    }

    /// change the target ifd in decoder
    pub fn change_ifd(&mut self, at: usize) -> Result<bool, DecodeError> {
        // If it already is, nothing will be done.
        if self.addr_index == at {
            return Ok(true);
        }

        let last_index = self.addrs.len() - 1;

        if last_index < at {
            let mut from = self.addrs[last_index];

            for _ in last_index..(at - 1) {
                let next = self.next_addr(from);

                if let Some(next) = next {
                    self.addrs.push(next);

                    // update
                    from = next;
                } else {
                    return Ok(false);
                }
            }
        }

        // No preblem, I'll update the index
        self.addr_index = at;

        // load ifd
        let addr = self.addrs[at];
        self.load_ifd(addr)?;

        // fin
        Ok(true)
    }

    fn next_addr(&mut self, from: u64) -> Option<u64> {
        self.reader.goto(from).ok()?;

        let entry_count = self.reader.read_u16(&self.endian).ok()?;

        // 2byte: tag id
        // 2byte: data type
        // 4byte: count field
        // 4byte: data field or pointer
        let skip_bytes = i64::from(entry_count * 12);
        self.reader.seek(SeekFrom::Current(skip_bytes)).ok()?;

        self.reader
            .read_u32(&self.endian)
            .ok()
            .map(|x| u64::from(x))
    }

    /// IFD constructor
    ///
    /// This function makes `ImageFileDirectory`
    ///
    /// ### figure of memory dump
    ///
    /// ```ignore
    ///                                                       +---- (4 byte) Entry.count (u32)
    ///                                                 +-----+---- (2 byte) Entry.datatype (`ifd::DataType`)
    ///                                           +-----+-----+---- (2 byte) Tag
    ///                                     +-----+-----+-----+---- (2 byte) Count of IFD entry (`ifd::Entry`)
    ///                   +-----------------+-----+-----+-----+---- (4 byte) Entry.offset ([u8; 4])
    ///                   |                 |     |     |     |
    ///                   |                 v     v     v     v
    /// 00000000 | -- --  v -- -- -- -- -- 00 10 FE 00 04 00 01 00
    /// 00000010 | 00 00 00 00 00 00 ...
    /// ```
    ///
    /// [`ifd`]: decode.Decoder.ifd
    fn load_ifd_with(
        mut reader: &mut R,
        endian: &Endian,
        from: u64,
    ) -> Result<ImageFileDirectory, DecodeError> {
        reader.goto(from)?;

        let entry_count = reader.read_u16(endian)?;
        let mut ifd = ImageFileDirectory::new();
        for _ in 0..entry_count {
            let tag = AnyTag::from_u16(reader.read_u16(endian)?);
            let ty = DataType::try_from(reader.read_u16(endian)?)?;
            let count = reader.read_u32(endian)? as usize;
            let field = reader.read_4byte()?;

            let entry = Entry::new(ty, count, field);
            ifd.insert_tag(tag, entry);
        }

        Ok(ifd)
    }

    fn load_ifd(&mut self, from: u64) -> Result<(), DecodeError> {
        let endian = self.endian().clone();
        let reader = self.reader_mut();

        self.ifd = Self::load_ifd_with(reader, &endian, from)?;

        Ok(())
    }

    fn get_elements<T: Tag>(
        &mut self,
        entry: Entry,
    ) -> Result<Vec<<T::Value as Decoded>::Element>, DecodingError> {
        let ty = entry.ty();
        let count = entry.count();
        let endian = self.endian().clone();

        let possible_count = <T::Value as Decoded>::POSSIBLE_COUNT;
        if !possible_count.contains_item(&count) {
            return Err(DecodingError::InvalidDataCount(count));
        }

        let mut elements = vec![];
        if entry.overflow() {
            let addr = self.reader.read_u32(&endian)?.into();
            self.reader.goto(addr)?;

            let reader = self.reader_mut();

            for _ in 0..count {
                let element = <T::Value as Decoded>::Element::read(reader, &endian, ty)?;
                elements.push(element);
            }
        } else {
            let mut reader = entry.field();

            for _ in 0..count {
                let element = <T::Value as Decoded>::Element::read(&mut reader, &endian, ty)?;
                elements.push(element);
            }
        }

        Ok(elements)
    }

    /// Get the `Tag::Value` in `ImageFileDirectory`.
    /// This function returns default value if T has default value and IFD doesn't have the value.
    pub fn get_value<T: Tag>(&mut self) -> Result<Option<T::Value>, DecodingError> {
        let entry = self.get_entry::<T>();

        match entry {
            Ok(Some(entry)) => {
                let entry = entry.clone();
                let elements = self.get_elements::<T>(entry)?;
                let val = <T::Value as Decoded>::decoded(elements)?;
                Ok(Some(val))
            }
            Ok(None) => Ok(T::DEFAULT_VALUE),
            Err(e) => Err(DecodingError::Tag(e.into_kind())),
        }
    }

    #[allow(missing_docs)]
    fn get_value_with<T: Tag>(
        &mut self,
        ifd: &ImageFileDirectory,
    ) -> Result<Option<T::Value>, DecodingError> {
        let entry = self.get_entry_with::<T>(ifd);

        match entry {
            Ok(Some(entry)) => {
                let entry = entry.clone();
                let elements = self.get_elements::<T>(entry)?;
                let val = <T::Value as Decoded>::decoded(elements)?;
                Ok(Some(val))
            }
            Ok(None) => Ok(T::DEFAULT_VALUE),
            Err(e) => Err(DecodingError::Tag(e.into_kind())),
        }
    }

    /// Get the `Tag::Value` in `ImageFileDirectory`.
    /// This function is almost the same as `Decoder::get_value`,
    /// but returns `DecodingError::NoValueThatShouldBe` if there is no value.
    /// If you want to use `Option` to get whether there is a value or not,
    /// you can use `Decoder::get_value`.
    pub fn get_exist_value<T: Tag>(&mut self) -> Result<T::Value, DecodingError> {
        let entry = self.get_entry::<T>();

        match entry {
            Ok(Some(entry)) => {
                let entry = entry.clone();
                let elements = self.get_elements::<T>(entry)?;
                let val = <T::Value as Decoded>::decoded(elements)?;
                Ok(val)
            }
            Ok(None) => T::DEFAULT_VALUE.ok_or(DecodingError::NoExistShouldExist),
            Err(e) => Err(DecodingError::Tag(e.into_kind())),
        }
    }

    #[allow(missing_docs)]
    fn get_exist_value_with<T: Tag>(
        &mut self,
        ifd: &ImageFileDirectory,
    ) -> Result<T::Value, DecodingError> {
        let entry = self.get_entry_with::<T>(ifd);

        match entry {
            Ok(Some(entry)) => {
                let entry = entry.clone();
                let elements = self.get_elements::<T>(entry)?;
                let val = <T::Value as Decoded>::decoded(elements)?;
                Ok(val)
            }
            Ok(None) => T::DEFAULT_VALUE.ok_or(DecodingError::NoExistShouldExist),
            Err(e) => Err(DecodingError::Tag(e.into_kind())),
        }
    }

    #[allow(missing_docs)]
    pub fn get_any_elements(&mut self, tag: AnyTag) -> DecodeResult<Option<AnyElements>> {
        let ifd = self.ifd();
        let entry = ifd.get_tag(&tag);

        if let Some(entry) = entry {
            let endian = self.endian().clone();
            let ty = entry.ty();
            let count = entry.count();
            let field = entry.field();

            if entry.overflow() {
                let addr = entry.field().read_u32(&endian)?.into();
                let reader = self.reader_mut();
                reader.goto(addr)?;

                let data = match ty {
                    DataType::Byte => read!(count, reader, read_u8, AnyElements::Byte, endian),
                    DataType::Ascii => read!(count, reader, read_ascii, AnyElements::Ascii, endian),
                    DataType::Short => read!(count, reader, read_u16, AnyElements::Short, endian),
                    DataType::Long => read!(count, reader, read_u32, AnyElements::Long, endian),
                    DataType::Rational => {
                        let vals = (0..count)
                            .into_iter()
                            .map(|_| {
                                let x = reader.read_u32(&endian);
                                let y = reader.read_u32(&endian);

                                x.and_then(|x| y.map(|y| Rational::new(x, y)))
                            })
                            .collect::<Result<Vec<_>, _>>()?;

                        AnyElements::Rational(vals)
                    }
                    DataType::SByte => read!(count, reader, read_i8, AnyElements::SByte, endian),
                    DataType::Undefined => {
                        read!(count, reader, read_u8, AnyElements::Undefined, endian)
                    }
                    DataType::SShort => read!(count, reader, read_i16, AnyElements::SShort, endian),
                    DataType::SLong => read!(count, reader, read_i32, AnyElements::SLong, endian),
                    DataType::SRational => {
                        let vals = (0..count)
                            .into_iter()
                            .map(|_| {
                                let x = reader.read_i32(&endian);
                                let y = reader.read_i32(&endian);

                                x.and_then(|x| y.map(|y| Rational::new(x, y)))
                            })
                            .collect::<Result<Vec<_>, _>>()?;

                        AnyElements::SRational(vals)
                    }
                    DataType::Float => read!(count, reader, read_f32, AnyElements::Float, endian),
                    DataType::Double => read!(count, reader, read_f64, AnyElements::Double, endian),
                };

                Ok(Some(data))
            } else {
                let mut reader = field;

                let data = match ty {
                    DataType::Byte => read!(count, reader, read_u8, AnyElements::Byte, endian),
                    DataType::Ascii => read!(count, reader, read_ascii, AnyElements::Ascii, endian),
                    DataType::Short => read!(count, reader, read_u16, AnyElements::Short, endian),
                    DataType::Long => read!(count, reader, read_u32, AnyElements::Long, endian),
                    DataType::Rational => {
                        let vals = (0..count)
                            .into_iter()
                            .map(|_| {
                                let x = reader.read_u32(&endian);
                                let y = reader.read_u32(&endian);

                                x.and_then(|x| y.map(|y| Rational::new(x, y)))
                            })
                            .collect::<Result<Vec<_>, _>>()?;

                        AnyElements::Rational(vals)
                    }
                    DataType::SByte => read!(count, reader, read_i8, AnyElements::SByte, endian),
                    DataType::Undefined => {
                        read!(count, reader, read_u8, AnyElements::Undefined, endian)
                    }
                    DataType::SShort => read!(count, reader, read_i16, AnyElements::SShort, endian),
                    DataType::SLong => read!(count, reader, read_i32, AnyElements::SLong, endian),
                    DataType::SRational => {
                        let vals = (0..count)
                            .into_iter()
                            .map(|_| {
                                let x = reader.read_i32(&endian);
                                let y = reader.read_i32(&endian);

                                x.and_then(|x| y.map(|y| Rational::new(x, y)))
                            })
                            .collect::<Result<Vec<_>, _>>()?;

                        AnyElements::SRational(vals)
                    }
                    DataType::Float => read!(count, reader, read_f32, AnyElements::Float, endian),
                    DataType::Double => read!(count, reader, read_f64, AnyElements::Double, endian),
                };

                Ok(Some(data))
            }
        } else {
            Ok(None)
        }
    }

    // pub fn image(&mut self) -> Result<ImageData, DecodeError> {
    //     let header = self.header();
    //     let width = header.width();
    //     let height = header.height();
    //     let bits_per_sample = header.bits_per_sample();
    //     let bits_len = bits_per_sample.len();
    //     let endian = self.endian().clone();

    //     let buffer_size = width * height * bits_len;
    //     let rows_per_strip = header.rows_per_strip();

    //     // let mut buffer = vec![];

    //     todo!()
    // }

    fn decode_bytes<D, W>(&mut self, mut decoder: D, mut writer: W) -> Result<usize, DecodingError>
    where
        D: DecodeBytes,
        W: io::Write,
    {
        let width = self.get_exist_value::<tag::ImageWidth>()?.as_size();
        let height = self.get_exist_value::<tag::ImageLength>()?.as_size();
        let rows_per_strip = self
            .get_value::<tag::RowsPerStrip>()?
            .map(|x| x.as_size())
            .unwrap_or(height);
        let bits_per_sample = self.get_value::<tag::BitsPerSample>()?;

        todo!()
    }
}

trait DecodeBytes {
    fn decode_bytes<R, W>(
        &mut self,
        reader: R,
        writer: W,
        compressed_length: usize,
        max_uncompressed_length: usize,
        predictor: Predictor,
    ) -> Result<usize, DecodingError>
    where
        R: io::Read,
        W: io::Write;
}

struct SimpleDecoder;

impl DecodeBytes for SimpleDecoder {
    fn decode_bytes<R, W>(
        &mut self,
        mut reader: R,
        mut writer: W,
        _compressed_length: usize,
        _max_uncompressed_length: usize,
        _predictor: Predictor,
    ) -> Result<usize, DecodingError>
    where
        R: io::Read,
        W: io::Write,
    {
        let copied_size = std::io::copy(&mut reader, &mut writer)?;

        usize::try_from(copied_size).map_err(|_| DecodingError::OverCapacity)
    }
}

struct LZWDecoder(weezl::decode::Decoder);

impl LZWDecoder {
    pub(crate) fn new() -> Self {
        let inner = weezl::decode::Decoder::with_tiff_size_switch(weezl::BitOrder::Msb, 8);

        return LZWDecoder(inner);
    }
}

impl DecodeBytes for LZWDecoder {
    fn decode_bytes<R, W>(
        &mut self,
        mut reader: R,
        mut writer: W,
        compressed_length: usize,
        max_uncompressed_length: usize,
        predictor: Predictor,
    ) -> Result<usize, DecodingError>
    where
        R: io::Read,
        W: io::Write,
    {
        let mut compressed_data = vec![0; compressed_length];
        reader.read_exact(&mut compressed_data[..])?;
        let mut uncompressed_data = Vec::with_capacity(max_uncompressed_length);

        let mut read = 0;
        while uncompressed_data.len() < max_uncompressed_length {
            let written = uncompressed_data.len();
            uncompressed_data.reserve(1 << 12);
            let buffer_space = uncompressed_data.capacity().min(max_uncompressed_length);
            uncompressed_data.resize(buffer_space, 0u8);

            let result = self
                .0
                .decode_bytes(&compressed_data[read..], &mut uncompressed_data[written..]);
            read += result.consumed_in;
            uncompressed_data.truncate(written + result.consumed_out);

            match result.status {
                Ok(weezl::LzwStatus::Ok) => {}
                Ok(weezl::LzwStatus::Done) => break,
                Ok(weezl::LzwStatus::NoProgress) => {
                    let err = io::Error::new(io::ErrorKind::UnexpectedEof, "no lzw end code found");
                    return Err(DecodingError::Io(err));
                }
                Err(err) => {
                    return Err(DecodingError::Io(io::Error::new(
                        io::ErrorKind::InvalidData,
                        err,
                    )))
                }
            }
        }

        uncompressed_data.shrink_to_fit();

        //

        writer.write_all(&uncompressed_data[..])?;

        let bytes = uncompressed_data.len();
        Ok(bytes)
    }
}

pub struct Addresses<'a, R> {
    addr: u64,
    idx: usize,
    endian: Endian,
    reader: &'a mut R,
}

impl<'a, R> Addresses<'a, R> {
    fn new(start: u64, endian: Endian, reader: &'a mut R) -> Self {
        Addresses {
            addr: start,
            idx: 0,
            endian,
            reader,
        }
    }
}

impl<'a, R> Iterator for Addresses<'a, R>
where
    R: io::Read + io::Seek,
{
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            Some(self.addr)
        } else {
            self.reader.goto(self.addr).ok()?;

            let entry_count = self.reader.read_u16(&self.endian).ok()?;

            // 2byte: tag id
            // 2byte: data type
            // 4byte: count field
            // 4byte: data field or pointer
            let skip_bytes = i64::from(entry_count * 12);
            self.reader.seek(SeekFrom::Current(skip_bytes)).ok()?;

            self.reader
                .read_u32(&self.endian)
                .ok()
                .map(|x| u64::from(x))
        }
    }
}
