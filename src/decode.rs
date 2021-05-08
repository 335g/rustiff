use crate::{data::{DataType, Entry}, element::{Elemental, Endian, EndianRead, SeekExt}, error::{DecodeError, DecodingError, FileHeaderError, TagError, TagErrorKind}, ifd::ImageFileDirectory, tag::{self, AnyTag, Tag}, val::{BitsPerSample, Compression, PhotometricInterpretation, Predictor, StripByteCounts, StripOffsets}};
use std::{convert::TryFrom, io, ops::RangeBounds};

pub trait Decoded: Sized {
    type Element: Elemental;
    type Poss: Possible;

    const POSSIBLE_COUNT: Self::Poss;

    fn decoded(elements: Vec<Self::Element>) -> Result<Self, DecodingError>;
}

pub trait Possible {
    fn contains_item(&self, item: &usize) -> bool;
}

impl Possible for usize {
    fn contains_item(&self, item: &usize) -> bool {
        *self == *item
    }
}

impl Possible for std::ops::Range<usize> {
    fn contains_item(&self, item: &usize) -> bool {
        self.contains(item)
    }
}

impl Possible for std::ops::RangeFrom<usize> {
    fn contains_item(&self, item: &usize) -> bool {
        self.contains(item)
    }
}

impl Possible for std::ops::RangeTo<usize> {
    fn contains_item(&self, item: &usize) -> bool {
        self.contains(item)
    }
}

impl Possible for std::ops::RangeFull {
    fn contains_item(&self, item: &usize) -> bool {
        self.contains(item)
    }
}

impl Possible for std::ops::RangeToInclusive<usize> {
    fn contains_item(&self, item: &usize) -> bool {
        self.contains(item)
    }
}

impl Possible for std::ops::RangeInclusive<usize> {
    fn contains_item(&self, item: &usize) -> bool {
        self.contains(item)
    }
}

impl<const N: usize> Possible for [usize; N] {
    fn contains_item(&self, item: &usize) -> bool {
        for i in self {
            if i == item {
                return true;
            }
        }

        false
    }
}

#[derive(Debug)]
enum Header {
    Pointer { at: u64 },
    Data { detail: HeaderDetail },
}

impl Header {
    #[allow(dead_code)]
    #[allow(missing_docs)]
    fn new(at: u64) -> Self {
        Header::Pointer { at }
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    fn unchecked_detail(&self) -> &HeaderDetail {
        match self {
            Header::Data { detail: x } => x,
            Header::Pointer { at: _ } => unreachable!(),
        }
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    fn unchecked_detail_into(self) -> HeaderDetail {
        match self {
            Header::Data { detail: x } => x,
            Header::Pointer { at: _ } => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct HeaderDetail {
    ifd: ImageFileDirectory,
    width: usize,
    height: usize,
    bits_per_sample: BitsPerSample,
    compression: Option<Compression>,
    photometric_interpretation: PhotometricInterpretation,
    rows_per_strip: usize,
    strip_offsets: StripOffsets,
    strip_byte_counts: StripByteCounts,
    predictor: Predictor,
}

impl HeaderDetail {
    #[inline]
    #[allow(dead_code)]
    #[allow(missing_docs)]
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
    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn endian(&self) -> &Endian {
        &self.endian
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub(crate) fn reader(&mut self) -> &mut R {
        &mut self.reader
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn width(&self) -> usize {
        self.headers[self.header_index].unchecked_detail().width
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn height(&self) -> usize {
        self.headers[self.header_index].unchecked_detail().height
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn bits_per_sample(&self) -> &BitsPerSample {
        &self.headers[self.header_index]
            .unchecked_detail()
            .bits_per_sample
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn compression(&self) -> Option<&Compression> {
        self.headers[self.header_index]
            .unchecked_detail()
            .compression
            .as_ref()
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn photometric_interpretation(&self) -> &PhotometricInterpretation {
        &self.headers[self.header_index]
            .unchecked_detail()
            .photometric_interpretation
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn rows_per_strip(&self) -> usize {
        self.headers[self.header_index]
            .unchecked_detail()
            .rows_per_strip
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn strip_byte_counts(&self) -> &StripByteCounts {
        &self.headers[self.header_index]
            .unchecked_detail()
            .strip_byte_counts
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn strip_offsets(&self) -> &StripOffsets {
        &self.headers[self.header_index]
            .unchecked_detail()
            .strip_offsets
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn predictor(&self) -> Predictor {
        self.headers[self.header_index].unchecked_detail().predictor
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn ifd(&self) -> &ImageFileDirectory {
        self.headers
            .get(self.header_index)
            .unwrap() // managing `ifd_index` with `ifds`, so there's always element.
            .unchecked_detail()
            .ifd()
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn get_entry<T: Tag>(&self) -> Result<Option<&Entry>, TagError<T>> {
        let ifd = self.ifd();

        self.get_entry_with::<T>(ifd)
    }

    #[allow(dead_code)]
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

        let entry = ifd.get_tag(anytag);
        Ok(entry)
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
    pub fn change_ifd(&mut self, at: usize) -> Result<(), DecodeError> {
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
    fn ifd_and_next_addr(&mut self, from: u64) -> Result<(ImageFileDirectory, u64), DecodeError> {
        let endian = self.endian().clone();
        let reader = self.reader();
        reader.goto(from)?;

        let entry_count = reader.read_u16(&endian)?;
        let mut ifd = ImageFileDirectory::new();
        for _ in 0..entry_count {
            let tag = AnyTag::from_u16(reader.read_u16(&endian)?);
            let ty = DataType::try_from(reader.read_u16(&endian)?)?;
            let count = reader.read_u32(&endian)? as usize;
            let field = reader.read_4byte()?;

            let entry = Entry::new(ty, count, field);
            ifd.insert_tag(tag, entry);
        }

        let next = self.reader.read_u32(&self.endian)?.into();

        Ok((ifd, next))
    }

    fn load_ifd(&mut self) -> Result<bool, DecodeError> {
        let last_index = self.headers.len() - 1;
        let last_header = self.headers.last().unwrap();
        let next_addr = match last_header {
            Header::Pointer { at: next_addr } => *next_addr,
            Header::Data { detail: _ } => {
                // reached the end
                return Ok(false)
            }
        };
        let (ifd, next_addr) = self.ifd_and_next_addr(next_addr)?;

        let width = self.get_exist_value_with::<tag::ImageWidth>(&ifd)?.as_size();
        let height = self.get_exist_value_with::<tag::ImageLength>(&ifd)?.as_size();
        let rows_per_strip = self.get_value_with::<tag::RowsPerStrip>(&ifd)?
            .map(|x| x.as_size())
            .unwrap_or(height);
        // let strip_offsets = self.get_exist_value_with::<tag::StripOffsets>(&ifd)?
        //     .map(|x| u64::from(x), |x| u64::from(x));
        let strip_offsets = self.get_exist_value_with::<tag::StripOffsets>(&ifd)?;
        let strip_byte_counts = self.get_exist_value_with::<tag::StripByteCounts>(&ifd)?;
        let bits_per_sample = self.get_exist_value_with::<tag::BitsPerSample>(&ifd)?;
        let compression = self.get_exist_value_with::<tag::Compression>(&ifd)?;
        let photometric_interpretation = self.get_exist_value_with::<tag::PhotometricInterpretation>(&ifd)?;
        let predictor = self.get_exist_value_with::<tag::Predictor>(&ifd)?;

        // // Each count must be equal.
        // if strip_offsets.len() != strip_byte_counts.len() {
        //     let infos = vec![
        //         (strip_offsets.len(), tag::StripOffsets::typename()),
        //         (strip_byte_counts.len(), tag::StripByteCounts::typename()),
        //     ];
        //     let err = DecodingError::InvalidCount(infos);

        //     return Err(DecodeError::from(err))
        // }

        // let header_detail = HeaderDetail {
        //     ifd, width, height, bits_per_sample, compression, photometric_interpretation, rows_per_strip, strip_offsets, strip_byte_counts, predictor
        // };

        // self.headers[last_index] = Header::Loaded {
        //     detail: header_detail,
        // };

        // append
        let next_header = Header::new(next_addr);
        self.headers.push(next_header);

        Ok(true)
    }

    fn get_elements<T: Tag>(&mut self, entry: Entry) -> Result<Vec<<T::Value as Decoded>::Element>, DecodingError> {
        let ty = entry.ty();
        let count = entry.count();
        let endian = self.endian().clone();

        let possible_count = <T::Value as Decoded>::POSSIBLE_COUNT;
        if !possible_count.contains_item(&count) {
            return Err(DecodingError::InvalidDataCount(count))
        }

        let one_size = <<T as Tag>::Value as Decoded>::Element::size(&ty);
        let total_size = one_size * count;

        if total_size > 4 {
            let addr = self.reader.read_u32(&endian)?;
            self.reader.goto(u64::from(addr))?;
        }

        let reader = self.reader();

        let mut elements = vec![];
        for _ in 0..count {
            let element = <T::Value as Decoded>::Element::read(reader, &endian, ty)?;
            elements.push(element);
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
    fn get_value_with<T: Tag>(&mut self, ifd: &ImageFileDirectory) -> Result<Option<T::Value>, DecodingError> {
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
    fn get_exist_value_with<T: Tag>(&mut self, ifd: &ImageFileDirectory) -> Result<T::Value, DecodingError> {
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
}
