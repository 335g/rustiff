use crate::{data::{Data, DataType, Entry}, num::DynamicTone, val::Predictor};
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
use crate::num::{Tone};
use byteorder::ByteOrder;
use tag::StripByteCounts;
use std::{any::type_name, convert::TryFrom};
use std::io::{self, Read};
use std::marker::PhantomData;
use std::{collections::HashSet, thread::current};

trait DecodeBytes {
    fn decode_bytes<R, W>(&mut self, reader: R, writer: W, compressed_length: usize, max_uncompressed_length: usize, predictor: Predictor) -> DecodeResult<usize>
    where
        R: io::Read,
        W: io::Write;
}

struct SimpleDecoder;

impl DecodeBytes for SimpleDecoder {
    fn decode_bytes<R, W>(&mut self, mut reader: R, mut writer: W, _compressed_length: usize, _max_uncompressed_length: usize, _predictor: Predictor) -> DecodeResult<usize>
    where
        R: io::Read,
        W: io::Write,
    {
        let copied_size = std::io::copy(&mut reader, &mut writer)?;
        let copied_size = usize::try_from(copied_size)
            .map_err(|_| DecodingError::UncompressedStripDataIsOverCapacity)?;

        Ok(copied_size)
    }
}

struct LZWDecoder(weezl::decode::Decoder);

impl LZWDecoder {
    pub(crate) fn new() -> Self {
        let inner = weezl::decode::Decoder::with_tiff_size_switch(weezl::BitOrder::Msb, 8);

        return LZWDecoder(inner)
    }
}

impl DecodeBytes for LZWDecoder {
    fn decode_bytes<R, W>(&mut self, mut reader: R, mut writer: W, compressed_length: usize, max_uncompressed_length: usize, predictor: Predictor) -> DecodeResult<usize>
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
            
            let result = self.0.decode_bytes(
                &compressed_data[read..],
                &mut uncompressed_data[written..],
            );
            read += result.consumed_in;
            uncompressed_data.truncate(written + result.consumed_out);
            
            match result.status {
                Ok(weezl::LzwStatus::Ok) => {}
                Ok(weezl::LzwStatus::Done) => break,
                Ok(weezl::LzwStatus::NoProgress) => {
                    let err = io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "no lzw end code found",
                    );
                    return Err(DecodeError::from(err))
                }
                Err(err) => return Err(DecodeError::from(io::Error::new(io::ErrorKind::InvalidData, err))),
            }
        }

        uncompressed_data.shrink_to_fit();

        // 

        writer.write_all(&uncompressed_data[..])?;

        let bytes = uncompressed_data.len();
        Ok(bytes)
    }
}

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
    width: usize,
    height: usize,
    bits_per_sample: BitsPerSample<DynamicTone>,
    compression: Option<Compression>,
    photometric_interpretation: PhotometricInterpretation,
    rows_per_strip: usize,
    strip_offsets: Vec<u64>,
    strip_byte_counts: Vec<usize>,
    predictor: Predictor,
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
    pub fn width(&self) -> usize {
        self.headers[self.header_index].unchecked_detail().width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.headers[self.header_index].unchecked_detail().height
    }

    #[inline]
    pub fn bits_per_sample(&self) -> &BitsPerSample<DynamicTone> {
        &self.headers[self.header_index].unchecked_detail().bits_per_sample
    }

    #[inline]
    pub fn compression(&self) -> Option<&Compression> {
        self.headers[self.header_index].unchecked_detail().compression.as_ref()
    }

    #[inline]
    pub fn photometric_interpretation(&self) -> &PhotometricInterpretation {
        &self.headers[self.header_index].unchecked_detail().photometric_interpretation
    }

    #[inline]
    pub fn rows_per_strip(&self) -> usize {
        self.headers[self.header_index].unchecked_detail().rows_per_strip
    }

    #[inline]
    pub fn strip_byte_counts(&self) -> &[usize] {
        self.headers[self.header_index].unchecked_detail().strip_byte_counts.as_slice()
    }

    #[inline]
    pub fn strip_offsets(&self) -> &[u64] {
        self.headers[self.header_index].unchecked_detail().strip_offsets.as_slice()
    }

    #[inline]
    pub fn predictor(&self) -> Predictor {
        self.headers[self.header_index].unchecked_detail().predictor
    }

    #[inline]
    pub fn ifd(&self) -> &ImageFileDirectory {
        self.headers
            .get(self.header_index)
            .unwrap() // managing `ifd_index` with `ifds`, so there's always element.
            .unchecked_detail()
            .ifd()
    }

    #[inline]
    #[allow(missing_docs)]
    pub fn get_entry<T: Tag>(&self) -> DecodeResult<Option<Entry>> {
        let ifd = self.ifd();
        
        self.get_entry_with::<T>(ifd)
    }

    #[inline]
    fn get_entry_with<T: Tag>(&self, ifd: &ImageFileDirectory) -> DecodeResult<Option<Entry>> {
        let anytag = AnyTag::try_from::<T>()?;

        let entry = ifd.get_tag(anytag).cloned();
        Ok(entry)
    }

    // #[allow(missing_docs)]
    // fn strip_count(&self) -> DecodeResult<usize> {
    //     let height = self.height() as usize;
    //     let rows_per_strip = self.rows_per_strip();

    //     if rows_per_strip == 0 {
    //         Ok(0)
    //     } else {
    //         Ok((height + rows_per_strip - 1) / rows_per_strip)
    //     }
    // }
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
        
        let width = self.get_exist_value_with::<tag::ImageWidth>(&ifd)?.as_size();
        let height = self.get_exist_value_with::<tag::ImageLength>(&ifd)?.as_size();
        let rows_per_strip = self.get_value_with::<tag::RowsPerStrip>(&ifd)?
            .map(|x| x.as_size())
            .unwrap_or(height);
        let strip_offsets = self.get_exist_value_with::<tag::StripOffsets>(&ifd)?
            .map(|x| u64::from(x), |x| u64::from(x));
        let strip_byte_counts = self.get_exist_value_with::<tag::StripByteCounts>(&ifd)?.as_size();
        let bits_per_sample = self.get_exist_value_with::<tag::BitsPerSample>(&ifd)?;
        let compression = self.get_exist_value_with::<tag::Compression>(&ifd)?;
        let photometric_interpretation = self.get_exist_value_with::<tag::PhotometricInterpretation>(&ifd)?;
        let predictor = self.get_exist_value_with::<tag::Predictor>(&ifd)?;

        // Each count must be equal.
        if strip_offsets.len() != strip_byte_counts.len() {
            let infos = vec![
                (strip_offsets.len(), tag::StripOffsets::typename()),
                (strip_byte_counts.len(), tag::StripByteCounts::typename()),
            ];
            let err = DecodingError::InvalidCount(infos);
            
            return Err(DecodeError::from(err))
        }

        let header_detail = HeaderDetail {
            ifd, width, height, bits_per_sample, compression, photometric_interpretation, rows_per_strip, strip_offsets, strip_byte_counts, predictor
        };
        
        self.headers[last_index] = Header::Loaded {
            detail: header_detail,
        };

        // append
        let next_header = Header::new(next_addr);
        self.headers.push(next_header);

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
            let count = reader.read_u32(&endian)? as usize;
            let field = reader.read_4byte()?;

            let entry = Entry::new(ty, count, field);
            ifd.insert_tag(tag, entry);
        }

        let next = self.reader.read_u32(&self.endian)?.into();

        Ok((ifd, next))
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

    #[allow(missing_docs)]
    fn get_value_with<T: Tag>(&mut self, ifd: &ImageFileDirectory) -> DecodeResult<Option<T::Value>> {
        let entry = self.get_entry_with::<T>(ifd);

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

    #[allow(missing_docs)]
    fn get_exist_value_with<T: Tag>(&mut self, ifd: &ImageFileDirectory) -> DecodeResult<T::Value> {
        let entry = self.get_entry_with::<T>(ifd);

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

    pub fn image(&mut self) -> DecodeResult<Data> {
        let width = self.width();
        let height = self.height();
        let bits_per_sample = self.bits_per_sample();
        let bits_len = bits_per_sample.len();
        let tone = bits_per_sample.tone().value();
        let endian = self.endian().clone();
        
        let buffer_size = width * height * bits_len;

        let rows_per_strip = self.rows_per_strip();
        let mut buffer = vec![];

        let loaded = match self.compression() {
            None => self.decode_bytes(SimpleDecoder, &mut buffer),
            Some(Compression::LZW) => self.decode_bytes(LZWDecoder::new(), &mut buffer)
        }?;

        // uncompression data length is not eq buffer size.
        if loaded * tone / 8 != buffer_size {
            let err = DecodingError::UnexpectedUncompressedSize {
                actual: loaded * tone / 8,
                required: buffer_size
            };

            return Err(DecodeError::from(err))
        }

        let data = match tone {
            8 => Data::U8(buffer),
            16 => {
                let buf = [0u8; 2];
                let (x1, x2): (Vec<_>, Vec<_>) = buffer.into_iter()
                    .enumerate()
                    .partition(|(i, _)| i % 2 == 0);
                
                let data = x1.into_iter()
                    .zip(x2)
                    .map(|((_, x1), (_, x2))| [x1, x2].as_ref().read_u16(&endian))
                    .collect::<Result<Vec<_>, _>>()?;

                Data::U16(data)
            }
            n => unreachable!("BitsPerSample is only available in 8 or 16 tones."),
        };

        return Ok(data);
    }

    fn decode_bytes<D, W>(&mut self, mut decoder: D, mut writer: W) -> DecodeResult<usize>
    where
        D: DecodeBytes,
        W: io::Write,
    {
        let width = self.width();
        let height = self.height();
        let rows_per_strip = self.rows_per_strip();
        let bits_len = self.bits_per_sample().len();
        let predictor = self.predictor();

        let values = self.strip_offsets()
            .iter()
            .zip(self.strip_byte_counts())
            .map(|(x, y)| (*x, *y))
            .enumerate()
            .collect::<Vec<_>>();

        let mut loaded = 0;
        for (i, (offset, byte_count)) in values {
            let strip_height = std::cmp::min(rows_per_strip, height - i * rows_per_strip);
            let buffer_size = width * strip_height * bits_len;
            
            self.reader().goto(offset)?;
            let mut buffer = vec![0u8; byte_count];
            self.reader().read_exact(&mut buffer[..])?;
            loaded += decoder.decode_bytes(buffer.as_slice(), &mut writer, byte_count, buffer_size, predictor)?;
        }

        Ok(loaded)
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
