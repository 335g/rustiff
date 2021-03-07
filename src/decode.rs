use crate::data::{Data, DataType, Entry};
use crate::dir::ImageFileDirectory;
use crate::error::{
    DecodeError, DecodeErrorKind, DecodeResult, DecodingError, FileHeaderError, TagError,
};
use crate::tag::{self, AnyTag, Tag};
use crate::val::{BitsPerSample, Compression, PhotometricInterpretation};
use crate::{
    byte::{Endian, EndianRead, SeekExt},
    dir,
};
use byteorder::ByteOrder;
use std::convert::TryFrom;
use std::io;
use std::marker::PhantomData;
use std::{collections::HashSet, thread::current};

pub trait Decoded: Sized {
    fn decode<'a, R: io::Read + io::Seek>(
        reader: &'a mut R,
        endian: &'a Endian,
        entry: Entry,
    ) -> DecodeResult<Self>;
}

#[derive(Debug)]
struct IFD {
    dir: Option<ImageFileDirectory>,
    at: u64,
}

impl IFD {
    fn new(at: u64) -> Self {
        IFD { dir: None, at }
    }
}

#[derive(Debug)]
pub struct Decoder<R> {
    reader: R,
    endian: Endian,
    ifd_index: usize,
    ifds: Vec<IFD>,
    // start_addresses: Vec<u64>,
}

impl<R> Decoder<R> {
    pub fn endian(&self) -> &Endian {
        &self.endian
    }

    pub(crate) fn reader(&mut self) -> &mut R {
        &mut self.reader
    }
}

impl<R> Decoder<R>
where
    R: io::Read + io::Seek,
{
    /// Constructor method
    ///
    /// ### errors
    ///
    /// This method occurs the error `error::FileHeaderErrorDetail`
    /// when file header is incorrect. This file header is 8 byte data
    /// before `IFD` part from the start.
    ///
    /// ### for_example
    ///
    /// ```ignore
    ///             +----------------(2 byte) Byte order (MM: Motorola type, II: Intel type)
    ///             |     +----------(2 byte) Version number (== 42)
    ///             |     |     +--- (4 byte) Pointer of IFD
    ///             |     |     |
    ///             v     v     v
    /// 00000000 | 49 49 2A 00 08 00 00 00 -- -- -- -- -- -- -- --
    /// 00000010 | -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
    /// ```
    pub fn new(mut reader: R) -> DecodeResult<Decoder<R>> {
        let mut byte_order = [0u8; 2];
        reader
            .read_exact(&mut byte_order)
            .map_err(|e| FileHeaderError::NoByteOrder)?;

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
        let ifds = vec![IFD::new(start)];

        let mut decoder = Decoder {
            reader,
            endian,
            ifd_index: 0,
            ifds,
        };

        // load the first ifd
        decoder.load_ifd()?;

        Ok(decoder)
    }

    /// change the target ifd in decoder
    pub fn change_ifd(&mut self, at: usize) -> DecodeResult<()> {
        // If it already is, nothing will be done.
        if self.ifd_index == at {
            return Ok(());
        }

        let last_index = self.ifds.len() - 1;

        if last_index < at {
            for i in last_index..(at - 1) {
                self.load_ifd()?;
            }

            self.load_ifd()?;
        }
        // No preblem, I'll update the index
        self.ifd_index = at;

        Ok(())
    }

    fn load_ifd(&mut self) -> DecodeResult<()> {
        let last_index = self.ifds.len() - 1;
        let last_ifd = self.ifds.last().unwrap();
        let next_addr = last_ifd.at;
        if next_addr == 0 || last_ifd.dir.is_some() {
            // reached the end
            return Err(DecodeError::from(
                DecodingError::CannotSelectImageFileDirectory,
            ));
        }

        let (current_ifd, next_addr) = self.ifd_and_next_addr(next_addr)?;
        let next_ifd = IFD::new(next_addr);

        let last_ifd = self.ifds.last_mut().unwrap();
        last_ifd.dir.replace(current_ifd);
        self.ifds.push(next_ifd);

        Ok(())
    }

    /// IFD constructor
    ///
    /// This function returns IFD and next IFD address.
    /// If you don't use multiple IFD, it's usually better to use [`ifd`] function.
    ///
    /// ### for_example
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
    fn ifd_and_next_addr(&mut self, from: u64) -> DecodeResult<(ImageFileDirectory, u64)> {
        let endian = self.endian().clone();
        let reader = self.reader();
        reader.goto(from)?;

        let entry_count = reader.read_u16(&endian)?;
        let mut ifd = ImageFileDirectory::new();
        for _ in 0..entry_count {
            let tag = AnyTag::from_u16(reader.read_u16(&endian)?);
            let ty = DataType::try_from(reader.read_u16(&endian)?)?;
            let count = reader.read_u32(&endian)?;
            let field = reader.read_4byte()?;

            let entry = Entry::new(ty, count, field);
            ifd.insert_tag(tag, entry);
        }

        let next = self.reader.read_u32(&self.endian)?.into();

        Ok((ifd, next))
    }

    #[inline]
    fn ifd(&self) -> DecodeResult<&ImageFileDirectory> {
        let ifd = self
            .ifds
            .get(self.ifd_index)
            .unwrap() // managing `ifd_index` with `ifds`, so there's always element.
            .dir
            .as_ref()
            .unwrap();

        Ok(ifd)
    }

    // /// `IFD` constructor
    // ///
    // /// Tiff file may have more than one `IFD`, but in most cases it is one and
    // /// you don't mind if you can access the first `IFD`. This function construct
    // /// the first `IFD`
    // fn ifd(&mut self) -> DecodeResult<ImageFileDirectory> {
    //     let (ifd, _) = self.ifd_and_next_addr(self.start)?;

    //     Ok(ifd)
    // }

    #[inline]
    #[allow(missing_docs)]
    fn get_entry<T: Tag>(&self) -> DecodeResult<Option<Entry>> {
        let ifd = self.ifd()?;
        let anytag = AnyTag::try_from::<T>()?;

        let entry = ifd.get_tag(anytag).cloned();
        Ok(entry)
    }

    /// Get the `Tag::Value` in `ImageFileDirectory`.
    /// This function returns default value if T has default value and IFD doesn't have the value.
    pub fn get_value<T: Tag>(&mut self) -> DecodeResult<Option<T::Value>> {
        let entry = self.get_entry::<T>();

        match entry {
            Ok(Some(entry)) => self.decode::<T::Value>(entry).map(|x| Some(x)),
            Ok(None) => Ok(T::DEFAULT_VALUE),
            Err(e) => Err(e),
        }
    }

    /// Get the `Tag::Value` in `ImageFileDirectory`.
    /// This function is almost the same as `Decoder::get_value`,
    /// but returns `DecodingError::NoValueThatShouldBe` if there is no value.
    /// If you want to use `Option` to get whether there is a value or not,
    /// you can use `Decoder::get_value`.
    pub fn get_exist_value<T: Tag>(&mut self) -> DecodeResult<T::Value> {
        let entry = self.get_entry::<T>();

        match entry {
            Ok(Some(entry)) => self.decode::<T::Value>(entry),
            Ok(None) => {
                T::DEFAULT_VALUE.ok_or(DecodeError::from(DecodingError::NoValueThatShouldBe))
            }
            Err(e) => Err(e),
        }
    }

    #[inline(always)]
    #[allow(missing_docs)]
    fn decode<D: Decoded>(&mut self, entry: Entry) -> DecodeResult<D> {
        D::decode(&mut self.reader, &self.endian, entry)
    }

    #[allow(missing_docs)]
    fn strip_count(&mut self) -> DecodeResult<u32> {
        let height = self.get_exist_value::<tag::ImageLength>()?.as_long();
        let rows_per_strip = self
            .get_value::<tag::RowsPerStrip>()?
            .map(|x| x.as_long())
            .unwrap_or_else(|| height);

        if rows_per_strip == 0 {
            Ok(0)
        } else {
            Ok((height + rows_per_strip - 1) / rows_per_strip)
        }
    }

    pub fn image(&mut self) -> DecodeResult<Data> {
        let width = self.get_exist_value::<tag::ImageWidth>()?.as_size();
        let height = self.get_exist_value::<tag::ImageLength>()?.as_size();
        let bits_per_sample = self.get_exist_value::<tag::BitsPerSample>()?;

        let buffer_size = width * height * bits_per_sample.len();

        let data = match bits_per_sample.max() {
            n if n <= 8 => Data::byte_with(buffer_size),
            n if n <= 16 => Data::short_with(buffer_size),
            n => return Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n]))),
        };

        // TODO: load data

        return Ok(data);
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;
    use std::{fs::File, io::stderr};

    use crate::tag;

    use super::Decoder;

    #[test]
    fn test() {
        // let f = File::open("tests/images/006_cmyk_tone_interleave_ibm_uncompressed.tif").expect("");
        let f = File::open("tests/images/010_cmyk_2layer.tif").expect("");
        let mut decoder = Decoder::new(f).expect("");

        // writeln!(&mut std::io::stderr(), "{}", decoder.start);
        // let (ifd1, start1) = decoder.ifd_and_next_addr(decoder.start).unwrap();

        // writeln!(&mut std::io::stderr(), "{}", start1);

        let ifd = decoder.ifd().unwrap();
        for tag in ifd.tags() {
            let _ = writeln!(&mut std::io::stderr(), "{:?}", tag);
        }
    }
}
