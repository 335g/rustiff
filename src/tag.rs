
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
use error::DecodeResult;

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
    ($($name:ident, $id:expr;)*) => {
        $(
        impl TagType for $name {
            type Value = u32;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<u32> { None }
            fn value_from(&self, from: Vec<u32>) -> DecodeResult<u32> {
                unimplemented!()
            }
        }
        )*
    };
}

macro_rules! tag_u32_with {
    ($($name:ident, $id:expr, $def:expr;)*) => {
        $(
        impl TagType for $name {
            type Value = u32;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<u32> { Some($def) }
            fn value_from(&self, from: Vec<u32>) -> DecodeResult<u32> {
                unimplemented!()
            }
        }
        )*
    };
}

macro_rules! tag_u16 {
    ($($name:ident, $id:expr;)*) => {
        $(
        impl TagType for $name {
            type Value = u16;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<u16> { None }
            fn value_from(&self, from: Vec<u32>) -> DecodeResult<u16> {
                unimplemented!()
            }
        }
        )*
    };
}

macro_rules! tag_u16_with {
    ($($name:ident, $id:expr, $def:expr;)*) => {
        $(
        impl TagType for $name {
            type Value = u16;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<u16> { Some($def) }
            fn value_from(&self, from: Vec<u32>) -> DecodeResult<u16> {
                unimplemented!()
            }
        }
        )*
    };
}

macro_rules! tag_vecu32 {
    ($($name:ident, $id:expr;)*) => {
        $(
        impl TagType for $name {
            type Value = Vec<u32>;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<Vec<u32>> { None }
            fn value_from(&self, from: Vec<u32>) -> DecodeResult<Vec<u32>> {
                Ok(from)
            }
        }
        )*
    };
}

macro_rules! tag_vecu32_with {
    ($($name:ident, $id:expr, $def:expr;)*) => {
        $(
        impl TagType for $name {
            type Value = Vec<u32>;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<Vec<u32>> { Some($def) }
            fn value_from(&self, from: Vec<u32>) -> DecodeResult<Vec<u32>> {
                Ok(from)
            }
        }
        )*
    };
}

macro_rules! tag_vecu8 {
    ($($name:ident, $id:expr;)*) => {
        $(
        impl TagType for $name {
            type Value = Vec<u8>;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<Vec<u8>> { None }
            fn value_from(&self, from: Vec<u32>) -> DecodeResult<Vec<u8>> {
                unimplemented!()
            }
        }
        )*
    };
}

macro_rules! tag_vecu8_with {
    ($($name:ident, $id:expr, $def:expr;)*) => {
        $(
        impl TagType for $name {
            type Value = Vec<u8>;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<Vec<u8>> { Some($def) }
            fn value_from(&self, from: Vec<u32>) -> DecodeResult<Vec<u8>> {
                unimplemented!()
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
    ImageWidth, 256;
    ImageLength, 257;
}

tag_u32_with! {
    RowsPerStrip, 278, u32::max_value();
}

tag_u16! {
    PhotometricInterpretation, 262;
}

tag_u16_with! {
    Compression, 259, 1;
    SamplesPerPixel, 277, 1;
}

tag_vecu8_with! {
    BitsPerSample, 258, vec![1];
}

tag_vecu32! {
    StripOffsets, 273;
    StripByteCounts, 279;
}

