use crate::{decode::Decoded, encode::Encoded, error::{TagError, TagErrorKind}, val};
use std::any::TypeId;
use std::fmt;

pub trait Tag: 'static {
    type Value: Decoded;
    type Elements: Encoded<Self::Value>;

    /// Default value when `ifd::IFD` doesn't have the value with this tag.
    const DEFAULT_VALUE: Option<Self::Value> = None;

    /// Identifer.
    ///
    /// This must not be equal to supported tag's identifer.
    /// If both identifer are equal, error occurs when you use this tag.
    const ID: u16;
}

macro_rules! define_tags {
    ($($name:ident,)*) => {
        /// Tag to get associated value from `ifd::ImageFileDirectory`.
        ///
        /// A tag that conforms to `tag::Tag` changes this automatically.
        #[non_exhaustive]
        #[derive(Debug, Clone, Hash, Eq, PartialEq)]
        pub enum AnyTag {
            $(
                #[allow(missing_docs)]
                $name,
            )*

            /// Unsupported tag.
            ///
            /// `rustiff` user can use unsupported tag to implement `tag::Tag`.
            /// This tag changes to `AnyTag::Custom` with `tag::Tag::ID`.
            Custom(u16),
        }

        impl AnyTag {
            /// Identifier
            ///
            /// Same as the value obtained by `tag::Tag::ID`.
            pub fn id(&self) -> u16 {
                match *self {
                    $(AnyTag::$name => $name::ID,)*
                    AnyTag::Custom(n) => n
                }
            }

            pub(crate) fn from_u16(n: u16) -> AnyTag {
                match n {
                    $($name::ID => AnyTag::$name,)*
                    _ => AnyTag::Custom(n),
                }
            }

            pub(crate) fn eq<T: Tag>(&self) -> bool {
                match *self {
                    $(AnyTag::$name => TypeId::of::<$name>() == TypeId::of::<T>(),)*
                    AnyTag::Custom(n) => n == T::ID,
                }
            }
            pub(crate) fn try_from<T: Tag>() -> Result<AnyTag, TagError<T>> {
                let anytag = AnyTag::from_u16(T::ID);

                if anytag.eq::<T>() {
                    Ok(anytag)
                } else {
                    // A tag with the same T::ID already exists.
                    let typename = std::any::type_name::<T>();
                    
                    let err = TagError::new(TagErrorKind::UnauthorizedTag {
                        tag_ty: typename
                    });

                    Err(err)
                }
            }
        }

        impl fmt::Display for AnyTag {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match *self {
                    $(AnyTag::$name => write!(f, "tag::{}", std::any::type_name::<$name>()),)*
                    AnyTag::Custom(n) => write!(f, "Unknown tag (id: {})", n),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ImageWidth {}

impl Tag for ImageWidth {
    type Value = val::ImageWidth;
    type Elements = val::Value;

    const ID: u16 = 256;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ImageLength {}

impl Tag for ImageLength {
    type Value = val::ImageLength;
    type Elements = val::Value;

    const ID: u16 = 257;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BitsPerSample {}

impl Tag for BitsPerSample {
    type Value = val::BitsPerSample;
    type Elements = Vec<u16>;

    const ID: u16 = 258;
    const DEFAULT_VALUE: Option<val::BitsPerSample> = Some(val::BitsPerSample::C1([1]));
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Compression {}

impl Tag for Compression {
    type Value = Option<val::Compression>;
    type Elements = u16;

    const ID: u16 = 259;
    const DEFAULT_VALUE: Option<Option<val::Compression>> = Some(None);
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PhotometricInterpretation {}

impl Tag for PhotometricInterpretation {
    type Value = val::PhotometricInterpretation;
    type Elements = u16;

    const ID: u16 = 262;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StripOffsets {}

impl Tag for StripOffsets {
    type Value = val::StripOffsets;
    type Elements = Vec<val::Value>;

    const ID: u16 = 273;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SamplesPerPixel {}

impl Tag for SamplesPerPixel {
    type Value = val::SamplesPerPixel;
    type Elements = u16;

    const ID: u16 = 277;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RowsPerStrip {}

impl Tag for RowsPerStrip {
    type Value = val::RowsPerStrip;
    type Elements = val::Value;

    const ID: u16 = 278;
    const DEFAULT_VALUE: Option<val::RowsPerStrip> = Some(val::RowsPerStrip::default_value());
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StripByteCounts {}

impl Tag for StripByteCounts {
    type Value = val::StripByteCounts;
    type Elements = Vec<val::Value>;

    const ID: u16 = 279;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Predictor {}

impl Tag for Predictor {
    type Value = val::Predictor;
    type Elements = u16;

    const ID: u16 = 317;

    const DEFAULT_VALUE: Option<Self::Value> = Some(val::Predictor::None);
}

define_tags! {
    ImageWidth,
    ImageLength,
    BitsPerSample,
    Compression,
    PhotometricInterpretation,
    StripOffsets,
    SamplesPerPixel,
    RowsPerStrip,
    StripByteCounts,
}
