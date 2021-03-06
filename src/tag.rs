
use std::fmt::{
    self,
    Display,
};
use std::io::{
    Read,
    Seek,
};
use error::{
    DecodeResult,
    DecodeError,
    DecodeErrorKind,
};
use ifd::DataType;
use byte::{
    Endian,
    EndianReadExt,
    SeekExt,
};

pub trait TagType: Clone + Copy {
    type Value;

    fn id(&self) -> u16;
    fn default_value() -> Option<Self::Value>;
    fn decode<'a, R: Read + Seek + 'a>(&'a self, reader: R, offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> DecodeResult<Self::Value>;
}

macro_rules! define_tags {
    ($($name:ident, $id:expr;)*) => {
        $(#[derive(Debug, Clone, Copy, PartialEq, Eq, Fail)]
        pub struct $name;

        impl Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", $name)
            }
        })*
        
        #[derive(Debug, Fail)]
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
        }

        impl Display for AnyTag {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match *self {
                    $(AnyTag::$name => $name.fmt(f),)*
                    AnyTag::Unknown(n) => write!(f, "Unknown tag: {}", n),
                }
            }
        }

        impl<T> From<T> for AnyTag where T: TagType {
            fn from(x: T) -> AnyTag {
                match x.id() {
                    $($id => AnyTag::$name,)*
                    _ => AnyTag::Unknown(x.id()),
                }
            }
        }

        impl From<u16> for AnyTag {
            fn from(n: u16) -> AnyTag {
                match n {
                    $($id => AnyTag::$name,)*
                    _ => AnyTag::Unknown(n),
                }
            }
        }
    }
}

macro_rules! tag_short_or_long_value {
    ($($name:ident, $id:expr, $def:expr;)*) => {
        $(impl TagType for $name {
            type Value = u32;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<u32> { $def }
            fn decode<'a, R: Read + Seek + 'a>(&'a self, mut _reader: R, mut offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> DecodeResult<Self::Value> {
                match datatype {
                    DataType::Short if count == 1 => Ok(offset.read_u16(endian)? as u32),
                    DataType::Long if count == 1 => Ok(offset.read_u32(endian)?),
                    _ => Err(DecodeError::from(DecodeErrorKind::NoSupportDataType { tag: AnyTag::from(*self), datatype: datatype, count: count })),
                }
            }
        })*
    };
}

macro_rules! tag_short_value {
    ($($name:ident, $id:expr, $def:expr;)*) => {
        $(impl TagType for $name {
            type Value = u16;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<u16> { $def }
            fn decode<'a, R: Read + Seek + 'a>(&'a self, mut _reader: R, mut offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> DecodeResult<Self::Value> {
                match datatype {
                    DataType::Short if count == 1 => Ok(offset.read_u16(endian)?),
                    _ => Err(DecodeError::from(DecodeErrorKind::NoSupportDataType { tag: AnyTag::from(*self), datatype: datatype, count: count })),
                }
            }
        })*
    };
}

macro_rules! tag_short_or_long_values {
    ($($name:ident, $id:expr, $def:expr;)*) => {
        $(impl TagType for $name {
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
                    _ => Err(DecodeError::from(DecodeErrorKind::NoSupportDataType { tag: AnyTag::from(*self), datatype: datatype, count: count })),
                }
            }
        })*
    };
}

macro_rules! tag_short_values {
    ($($name:ident, $id:expr, $def:expr;)*) => {
        $(impl TagType for $name {
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
                    _ => Err(DecodeError::from(DecodeErrorKind::NoSupportDataType { tag: AnyTag::from(*self), datatype: datatype, count: count })),
                }
            }
        })*
    };
}

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

tag_short_or_long_value! {
    ImageWidth, 256, None;
    ImageLength, 257, None;
    RowsPerStrip, 278, Some(u32::max_value());
}

tag_short_or_long_values! {
    StripOffsets, 273, None;
    StripByteCounts, 279, None;
}

tag_short_value! {
    PhotometricInterpretation, 262, None;
    Compression, 259, Some(1);
    SamplesPerPixel, 277, Some(1);
}

tag_short_values! {
    BitsPerSample, 258, Some(vec![1]);
}


