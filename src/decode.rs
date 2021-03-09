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
enum Header {
    Unloaded { at: u64 },
    Loaded { detail: HeaderDetail },
}

impl Header {
    fn new(at: u64) -> Self {
        Header::Unloaded { at }
    }

    fn unchecked_detail(&self) -> &HeaderDetail {
        match self {
            Header::Loaded { detail: x } => x,
            Header::Unloaded { at: _ } => unreachable!(),
        }
    }

    fn unchecked_detail_into(self) -> HeaderDetail {
        match self {
            Header::Loaded { detail: x } => x,
            Header::Unloaded { at: _ } => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct HeaderDetail {
    ifd: ImageFileDirectory,
    width: u32,
    height: u32,
}

impl HeaderDetail {
    fn ifd(&self) -> &ImageFileDirectory {
        &self.ifd
    }
}

#[derive(Debug)]
pub struct Decoder<R> {
    reader: R,
    endian: Endian,
    header_index: usize,
    headers: Vec<Header>,
}

impl<R> Decoder<R> {
    pub fn endian(&self) -> &Endian {
        &self.endian
    }

    pub(crate) fn reader(&mut self) -> &mut R {
        &mut self.reader
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.headers[self.header_index].unchecked_detail().width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.headers[self.header_index].unchecked_detail().height
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
        let headers = vec![Header::new(start)];

        let mut decoder = Decoder {
            reader,
            endian,
            header_index: 0,
            headers,
        };

        // load the first ifd
        decoder.load_ifd()?;

        Ok(decoder)
    }

    /// change the target ifd in decoder
    pub fn change_ifd(&mut self, at: usize) -> DecodeResult<()> {
        // If it already is, nothing will be done.
        if self.header_index == at {
            return Ok(());
        }

        let last_index = self.headers.len() - 1;

        if last_index < at {
            for i in last_index..(at - 1) {
                self.load_ifd()?;
            }

            self.load_ifd()?;
        }
        // No preblem, I'll update the index
        self.header_index = at;

        Ok(())
    }

    fn load_ifd(&mut self) -> DecodeResult<()> {
        let last_index = self.headers.len() - 1;
        let last_header = self.headers.last().unwrap();
        let next_addr = match last_header {
            Header::Unloaded { at: next_addr } => *next_addr,
            Header::Loaded { detail: _ } => {
                // reached the end
                return Err(DecodeError::from(
                    DecodingError::CannotSelectImageFileDirectory,
                ));
            }
        };
        let (ifd, next_addr) = self.ifd_and_next_addr(next_addr)?;

        // tmp update, because cannot load ifd.
        let header_detail = HeaderDetail {
            ifd,
            width: 0,
            height: 0,
        };
        self.headers[last_index] = Header::Loaded {
            detail: header_detail,
        };

        // append
        let next_header = Header::new(next_addr);
        self.headers.push(next_header);

        // true update
        let width = self.get_exist_value::<tag::ImageWidth>()?.as_long();
        let height = self.get_exist_value::<tag::ImageLength>()?.as_long();
        match self.headers.get_mut(last_index).unwrap() {
            Header::Loaded { detail } => {
                detail.width = width;
                detail.height = height;
            }
            _ => unreachable!(),
        }

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
            .headers
            .get(self.header_index)
            .unwrap() // managing `ifd_index` with `ifds`, so there's always element.
            .unchecked_detail()
            .ifd();

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
    use std::{fmt::write, io::Write};
    use std::{fs::File, io::stderr};

    use crate::tag;

    use super::Decoder;

    #[test]
    fn test() {
        // let f = File::open("tests/images/006_cmyk_tone_interleave_ibm_uncompressed.tif").expect("");
        let f = File::open("tests/images/010_cmyk_2layer.tif").expect("");
        let mut decoder = Decoder::new(f).expect("");

        // let width = decoder.get_exist_value::<tag::ImageWidth>().map(|x| x.as_long());
        // let height = decoder.get_exist_value::<tag::ImageLength>().map(|x| x.as_long());

        // let mut err = &mut std::io::stderr();
        // writeln!(&mut err, "width: {:?}", width);
        // writeln!(&mut err, "height: {:?}", height);
    }
}
