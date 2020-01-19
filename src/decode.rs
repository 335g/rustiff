use crate::byte::{Endian, EndianRead, SeekExt};
use crate::error::{
    DecodeError, DecodeErrorKind, DecodeResult, DecodeValueErrorDetail, FileHeaderErrorDetail,
    TagErrorKind,
};
use crate::ifd::{DataType, Entry, IFD};
use crate::tag::{AnyTag, Tag};
use crate::val::{BitsPerSample, Compression, PhotometricInterpretation};
use byteorder::ByteOrder;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::io::{self, Read, Seek};
use std::marker::PhantomData;

pub trait DecodeFrom: Sized {
    fn decode<R: Read + Seek>(decoder: &mut Decoder<R>, entry: &Entry) -> DecodeResult<Self>;
}

#[derive(Debug)]
pub struct Decoder<R> {
    reader: R,
    endian: Endian,
    start: u64,
    next: u64,
}

impl<R> Decoder<R> {
    pub fn endian(&self) -> Endian {
        self.endian
    }
}

impl<R> Decoder<R>
where
    R: Read + Seek,
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
            .map_err(|e| FileHeaderErrorDetail::NoByteOrder)?;

        let endian = match &byte_order {
            b"II" => Endian::Little,
            b"MM" => Endian::Big,
            _ => {
                return Err(DecodeError::from(FileHeaderErrorDetail::InvalidByteOrder {
                    byte_order: byte_order,
                }))
            }
        };

        let _ = reader
            .read_u16(endian)
            .map_err(|_| FileHeaderErrorDetail::NoVersion)
            .and_then(|n| {
                if n == 42 {
                    Ok(())
                } else {
                    Err(FileHeaderErrorDetail::InvalidVersion { version: n })
                }
            })?;

        let start: u64 = reader
            .read_u32(endian)
            .map_err(|_| FileHeaderErrorDetail::NoIFDAddress)?
            .into();
        let next = start;

        Ok(Decoder {
            reader,
            endian,
            start,
            next,
        })
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
    pub fn ifd_and_next_addr(&mut self, from: u64) -> DecodeResult<(IFD, u64)> {
        self.reader.goto(from)?;
        let endian = self.endian;

        let mut ifd = IFD::new();
        for _ in 0..self.reader.read_u16(endian)? {
            let tag = AnyTag::from_u16(self.reader.read_u16(endian)?);
            let ty = DataType::try_from(self.reader.read_u16(endian)?)?;
            let count = self.reader.read_u32(endian)?;
            let field = self.reader.read_4byte()?;

            let entry = Entry::new(ty, count, field);
            ifd.insert_tag(tag, entry);
        }

        let next = self.read_u32(endian)?.into();

        Ok((ifd, next))
    }

    /// `IFD` constructor
    ///
    /// Tiff file may have more than one `IFD`, but in most cases it is one and
    /// you don't mind if you can access the first `IFD`. This function construct
    /// the first `IFD`
    pub fn ifd(&mut self) -> DecodeResult<IFD> {
        let (ifd, _) = self.ifd_and_next_addr(self.start)?;

        Ok(ifd)
    }

    #[inline]
    #[allow(missing_docs)]
    fn get_entry<'a, T: Tag>(&mut self, ifd: &'a IFD, tag: T) -> DecodeResult<&'a Entry> {
        let anytag = AnyTag::try_from::<T>()?;

        let entry = ifd
            .get_tag(anytag)
            .ok_or(TagErrorKind::cannot_find_tag::<T>())?;
        Ok(entry)
    }

    /// Get the `Tag::Value` in `IFD`.
    pub fn get_value<T: Tag>(&mut self, ifd: &IFD, tag: T) -> DecodeResult<T::Value> {
        let entry = self.get_entry(ifd, tag)?;
        self.decode::<T::Value>(entry)
    }

    #[inline]
    #[allow(missing_docs)]
    fn decode<D: DecodeFrom>(&mut self, entry: &Entry) -> DecodeResult<D> {
        D::decode(self, entry)
    }

    #[allow(missing_docs)]
    fn strip_count(&mut self) -> DecodeResult<u32> {
        unimplemented!()
    }
}

impl<S> Seek for Decoder<S>
where
    S: Seek,
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}

impl<R> Read for Decoder<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}
