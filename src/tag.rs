
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use failure::{
    Fail,
};
use std::fmt::{
    self,
    Display,
};
use error::{
    DecodeResult,
    DecodeError,
    DecodeErrorKind,
};

pub trait TagType: Clone + Copy {
    type Value;

    fn id(&self) -> u16;
    fn default_value() -> Option<Self::Value>;
    fn value_from(&self, from: Vec<u32>) -> DecodeResult<Self::Value>;
}

macro_rules! define_tags {
    ($($name:ident, $id:expr;)*) => {
        $(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Fail)]
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
            pub fn from_u16(x: u16) -> AnyTag {
                match x {
                    $($id => AnyTag::$name,)*
                    _ => AnyTag::Unknown(x),
                }
            }

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
    }
}

macro_rules! tag_u32 {
    ($($name:ident, $id:expr, $def:expr;)*) => {
        $(
        impl TagType for $name {
            type Value = u32;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<u32> { $def }
            fn value_from(&self, from: Vec<u32>) -> DecodeResult<u32> {
                match from.len() {
                    1 => Ok(from[0]),
                    0 => Err(DecodeError::from(DecodeErrorKind::NoData { tag: AnyTag::$name })),
                    _ => Err(DecodeError::from(DecodeErrorKind::ExtraData { tag: AnyTag::$name, data: from })),
                }
            }
        }
        )*
    };
}

macro_rules! tag_u16 {
    ($($name:ident, $id:expr, $def:expr;)*) => {
        $(
        impl TagType for $name {
            type Value = u16;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<u16> { $def }
            fn value_from(&self, from: Vec<u32>) -> DecodeResult<u16> {
                match from.len() {
                    1 => {
                        let from_value = from[0];
                        let max_value = u16::max_value();
                        if from_value > max_value as u32 {
                            Err(DecodeError::from(DecodeErrorKind::OverflowU16Data { tag: AnyTag::$name, data: from_value }))
                        } else {
                            Ok(from_value as u16)
                        }
                    },
                    0 => Err(DecodeError::from(DecodeErrorKind::NoData { tag: AnyTag::$name })),
                    _ => Err(DecodeError::from(DecodeErrorKind::ExtraData { tag: AnyTag::$name, data: from })),
                }
            }
        }
        )*
    };
}

macro_rules! tag_vecu32 {
    ($($name:ident, $id:expr, $def:expr;)*) => {
        $(
        impl TagType for $name {
            type Value = Vec<u32>;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<Vec<u32>> { $def }
            fn value_from(&self, from: Vec<u32>) -> DecodeResult<Vec<u32>> {
                match from.len() {
                    0 => Err(DecodeError::from(DecodeErrorKind::NoData { tag: AnyTag::$name })),
                    _ => Ok(from),
                }
            }
        }
        )*
    };
}

macro_rules! tag_vecu8 {
    ($($name:ident, $id:expr, $def:expr;)*) => {
        $(
        impl TagType for $name {
            type Value = Vec<u8>;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<Vec<u8>> { $def }
            fn value_from(&self, from: Vec<u32>) -> DecodeResult<Vec<u8>> {
                match from.len() {
                    0 => Err(DecodeError::from(DecodeErrorKind::NoData { tag: AnyTag::$name })),
                    _ => {
                        let max_value = u8::max_value();
                        let mut from_values: Vec<u8> = vec![];
                        for from_value in from {
                            if from_value > max_value as u32 {
                                return Err(DecodeError::from(DecodeErrorKind::OverflowU16Data { tag: AnyTag::$name, data: from_value }));
                            }
                            from_values.push(from_value as u8);
                        }

                        Ok(from_values)
                    }
                }
            }
        }
        )*
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

tag_u32! {
    ImageWidth, 256, None;
    ImageLength, 257, None;
    RowsPerStrip, 278, Some(u32::max_value());
}

tag_u16! {
    PhotometricInterpretation, 262, None;
    Compression, 259, Some(1);
    SamplesPerPixel, 277, Some(1);
}

tag_vecu8! {
    BitsPerSample, 258, Some(vec![1]);
}

tag_vecu32! {
    StripOffsets, 273, None;
    StripByteCounts, 279, None;
}

