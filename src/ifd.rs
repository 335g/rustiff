
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::HashMap;

macro_rules! tags {
    {$($tag:ident $val:expr;)*} => {
        #[derive(Debug, Clone, Hash, Eq, PartialEq)]
        pub enum Tag {
            $($tag,)*
            Unknown(u16),
        }

        impl Tag {
            pub fn from_u16(n: u16) -> Tag {
                $(if n == $val {
                    Tag::$tag
                } else)* {
                    Tag::Unknown(n)
                }
            }
        }
    }
}

tags!{
    // must have
    ImageWidth 256;
    ImageLength 257;
    BitsPerSample 258;
    Compression 259;
    PhotometricInterpretation 262;
    StripOffsets 273;
    RowsPerStrip 278;
    StripByteCounts 279;
    XResolusion 282;
    YResolusion 283;
    ResolutionUnit 296;
    ColorMap 320;
    
    // option TODO: more tags
}

#[derive(Debug, Clone)]
pub enum DataType {
    Byte,
    Ascii,
    Short,
    Long,
    Rational,
}

#[derive(Debug, Clone)]
pub struct Entry {
    date_type: DataType,
    count: u32,
    offset: [u8; 4],
}

pub type IFD = HashMap<Tag, Entry>;

