
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

macro_rules! tags {
    {$($tag:ident $val:expr;)*} => {
        #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
        pub enum TagKind {
            $($tag,)*
            Unknown(u16),
        }

        impl TagKind {
            pub fn from_u16(n: u16) -> TagKind {
                $(if n == $val {
                    TagKind::$tag
                } else)* {
                    TagKind::Unknown(n)
                }
            }

            pub fn all() -> Vec<TagKind> {
                vec![
                    $(TagKind::$tag,)*
                ]
            }
        }
    }
}

tags!{
    ImageWidth 256;
    ImageLength 257;
    BitsPerSample 258;
    Compression 259;
    PhotometricInterpretation 262;
    StripOffsets 273;
    SamplesPerPixel 277;
    RowsPerStrip 278;
    StripByteCounts 279;
    XResolusion 282;
    YResolusion 283;
    ResolutionUnit 296;
}
