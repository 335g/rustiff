
use std::fmt::{
    self,
    Display,
};
use std::io::{
    Read,
    Seek,
};
use std::any::TypeId;
use error::{
    DecodeResult,
    DecodeError,
    DecodeErrorKind,
};
use ifd::DataType;
use byte::{
    Endian,
    ReadExt,
    SeekExt,
};
use failure::Fail;

pub trait TagType: fmt::Debug + Display + Send + Sync + 'static + Clone + Copy {
    type Value: fmt::Debug + Send + Sync;

    fn id(&self) -> u16;
    fn default_value() -> Option<Self::Value>;
    fn decode<'a, R: Read + Seek + 'a>(&'a self, reader: R, offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> DecodeResult<Self::Value>;
}

#[derive(Debug, Clone)]
//#[fail(display = "Impossible tag: {}", _0)]
pub struct ImpossibleTag<T: TagType>(T);

impl<T> ImpossibleTag<T> where T: TagType {
    pub fn tag(self) -> T {
        self.0
    }
}

macro_rules! define_tags {
    ($($name:ident, $id:expr;)*) => {
        $(impl Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "tag::{}", $name)
            }
        })*
        
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub enum AnyTag {
            $($name,)*
            Unknown(u16),
        }

        impl AnyTag {
            pub fn id(&self) -> u16 {
                match *self {
                    $(AnyTag::$name => $id,)*
                    AnyTag::Unknown(n) => n,
                }
            }

            ///
            pub(crate) fn from_u16(n: u16) -> AnyTag {
                match n {
                    $($id => AnyTag::$name,)*
                    _ => AnyTag::Unknown(n),
                }
            }

        }

        impl Display for AnyTag {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match *self {
                    $(AnyTag::$name => $name.fmt(f),)*
                    AnyTag::Unknown(n) => write!(f, "Unknown tag: {}", n),
                }
            }
        }

        impl<T> PartialEq<T> for AnyTag where T: TagType {
            fn eq(&self, rhs: &T) -> bool {
                match *self {
                    $(AnyTag::$name => TypeId::of::<$name>() == TypeId::of::<T>(),)*
                    AnyTag::Unknown(n) => n == rhs.id(),
                    _ => false
                }
            }
        }
    }
}

impl AnyTag {
    #[inline]
    pub(crate) fn try_from<T>(tag: T) -> Result<AnyTag, ImpossibleTag<T>> where T: TagType {
        let anytag = AnyTag::from_u16(tag.id());
        if anytag == tag {
            Ok(anytag)
        } else {
            Err(ImpossibleTag(tag))
        }
    }
}

macro_rules! short_or_long_value {
    ($name:ident, $id:expr, $def:expr) => {
        impl TagType for $name {
            type Value = u32;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<u32> { $def }
            fn decode<'a, R: Read + Seek + 'a>(&'a self, mut _reader: R, mut offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> DecodeResult<Self::Value> {
                match datatype {
                    DataType::Short if count == 1 => Ok(offset.read_u16(endian)? as u32),
                    DataType::Long if count == 1 => Ok(offset.read_u32(endian)?),
                    _ => Err(DecodeError::from(DecodeErrorKind::UnsupportedDataTypeAndCount { tag: AnyTag::try_from(*self)?, datatype: datatype, count: count })),
                }
            }
        }
    }
}

macro_rules! short_value {
    ($name:ident, $id:expr, $def:expr) => {
        impl TagType for $name {
            type Value = u16;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<u16> { $def }
            fn decode<'a, R: Read + Seek + 'a>(&'a self, mut _reader: R, mut offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> DecodeResult<Self::Value> {
                match datatype {
                    DataType::Short if count == 1 => Ok(offset.read_u16(endian)?),
                    _ => Err(DecodeError::from(DecodeErrorKind::UnsupportedDataTypeAndCount { tag: AnyTag::try_from(*self)?, datatype: datatype, count: count })),
                }
            }
        }
    }
}

macro_rules! short_or_long_values {
    ($name:ident, $id:expr, $def:expr) => {
        impl TagType for $name {
            type Value = Vec<u32>;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<Vec<u32>> { $def }
            fn decode<'a, R: Read + Seek + 'a>(&'a self, mut reader: R, mut offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> DecodeResult<Self::Value> {
                match datatype {
                    DataType::Short if count == 1 => Ok(vec![offset.read_u16(endian)? as u32]),
                    DataType::Short if count == 2 => Ok(vec![
                        offset.read_u16(endian)? as u32,
                        offset.read_u16(endian)? as u32,
                    ]),
                    DataType::Short if count > 2 => {
                        let offset = offset.read_u32(endian)? as u64;
                        reader.goto(offset)?;
                        let mut v = Vec::with_capacity(count);
                        for _ in 0..count {
                            v.push(reader.read_u16(endian)? as u32);
                        }

                        Ok(v)
                    }
                    DataType::Long if count == 1 => Ok(vec![offset.read_u32(endian)?]),
                    DataType::Long if count > 1 => {
                        let offset = offset.read_u32(endian)? as u64;
                        reader.goto(offset)?;
                        let mut v = Vec::with_capacity(count);
                        for _ in 0..count {
                            v.push(reader.read_u32(endian)?);
                        }

                        Ok(v)
                    }
                    _ => Err(DecodeError::from(DecodeErrorKind::UnsupportedDataTypeAndCount { tag: AnyTag::try_from(*self)?, datatype: datatype, count: count })),
                }
            }
        }
    }
}

macro_rules! short_values {
    ($name:ident, $id:expr, $def:expr) => {
        impl TagType for $name {
            type Value = Vec<u16>;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<Vec<u16>> { $def }
            fn decode<'a, R: Read + Seek + 'a>(&'a self, mut reader: R, mut offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> DecodeResult<Self::Value> {
                match datatype {
                    DataType::Short if count == 1 => Ok(vec![offset.read_u16(endian)?]),
                    DataType::Short if count == 2 => Ok(vec![
                        offset.read_u16(endian)?,
                        offset.read_u16(endian)?,
                    ]),
                    DataType::Short if count > 2 => {
                        let offset = offset.read_u32(endian)? as u64;
                        reader.goto(offset)?;
                        let mut v = Vec::with_capacity(count);
                        for _ in 0..count {
                            v.push(reader.read_u16(endian)?);
                        }

                        Ok(v)
                    }
                    _ => Err(DecodeError::from(DecodeErrorKind::UnsupportedDataTypeAndCount { tag: AnyTag::try_from(*self)?, datatype: datatype, count: count })),
                }
            }
        }
    }
}

/// The number of columns in the image.
///
/// 
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct ImageWidth;
short_or_long_value!(ImageWidth, 256, None);

/// The number of rows in the image.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct ImageLength;
short_or_long_value!(ImageLength, 257, None);

/// 
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct BitsPerSample;
short_values!(BitsPerSample, 258, Some(vec![1]));

/// 
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct Compression;
short_value!(Compression, 259, Some(1));

/// 
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct PhotometricInterpretation;
short_value!(PhotometricInterpretation, 262, None);

/// 
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct StripOffsets;
short_or_long_values!(StripOffsets, 273, None);

/// 
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct SamplesPerPixel;
short_value!(SamplesPerPixel, 277, Some(1));

/// 
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct RowsPerStrip;
short_or_long_value!(RowsPerStrip, 278, Some(u32::max_value()));

///
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct StripByteCounts;
short_or_long_values!(StripByteCounts, 279, None);


define_tags! {
    ImageWidth, 256;
    ImageLength, 257;
    BitsPerSample, 258;
    Compression, 259;
    PhotometricInterpretation, 262;
    StripOffsets, 273;
    SamplesPerPixel, 277;
    RowsPerStrip, 278;
    StripByteCounts, 279;
}

