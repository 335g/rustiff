use crate::{decode::Decoded};
use crate::error::{DecodeError, DecodeErrorKind, DecodeResult, TagError, TagErrorKind};
use crate::val::{self, Long, Short, Value, Values};
use crate::num::{Tone, DynamicTone, T1};
use std::{any::TypeId, marker::PhantomData};
use std::fmt;

pub trait Tag: 'static {
    /// The value associated with this tag.
    type Value: Decoded;

    /// Identifer.
    ///
    /// This must not be equal to supported tag's identifer.
    /// If both identifer are equal, error occurs when you use this tag.
    const ID: u16;

    /// Default value when `ifd::IFD` doesn't have the value with this tag.
    const DEFAULT_VALUE: Option<Self::Value> = None;

    fn typename() -> &'static str {
        std::any::type_name::<Self>()
    }
}

macro_rules! define_tags {
    ($($name:ident,)*) => {
        /// Tag to get associated value from `ifd::IFD`.
        ///
        /// A tag that conforms to `tag::TagType` changes this automatically.
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
            /// This tag chnages to `AnyTag::Custom` with `tag::Tag::ID`.
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

            /// Constructor
            pub(crate) fn from_u16(n : u16) -> AnyTag {
                match n {
                    $($name::ID => AnyTag::$name,)*
                    _ => AnyTag::Custom(n),
                }
            }

            pub(crate) fn try_from<T: Tag>() -> DecodeResult<AnyTag> {
                let anytag = AnyTag::from_u16(T::ID);

                if anytag.eq::<T>() {
                    Ok(anytag)
                } else {
                    // A tag with the same T::ID already exists.
                    let typename = std::any::type_name::<T>().to_string();
                    let err = TagError::new(TagErrorKind::UnauthorizedTag(typename));
                    Err(DecodeError::from(err))
                }
            }

            pub(crate) fn eq<T: Tag>(&self) -> bool {
                match *self {
                    $(AnyTag::$name => TypeId::of::<$name>() == TypeId::of::<T>(),)*
                    AnyTag::Custom(n) => n == T::ID,
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

/// The tag for the number of columns of pixels in the image.
///
/// ### code_id
///
/// 256
///
/// ### datatype
///
/// - [`Short`]
/// - [`Long`]
///
/// ### count
///
/// 1
///
/// ### default_value
///
/// None
///
/// [`Short`]: ifd.DataType.html
/// [`Long`]: ifd.DataType.html
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ImageWidth {}

impl Tag for ImageWidth {
    type Value = Value;

    const ID: u16 = 256;
}

/// The tag for the number of rows of pixels in the image.
///
/// ### code_id
///
/// 257
///
/// ### datatype
///
/// - [`Short`]
/// - [`Long`]
///
/// ### count
///
/// 1
///
/// ### default_value
///
/// None
///
/// [`Short`]: ifd.DataType.html
/// [`Long`]: ifd.DataType.html
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ImageLength {}

impl Tag for ImageLength {
    type Value = Value;

    const ID: u16 = 257;
}

/// The tag for the number of bits per component.
///
/// This field allows a different number of bits per component for each component corresponding a pixel.
/// For example, RGB color data could use a different number of bits per component for each of the three
/// color planes. Most RGB files will have the same number of BitsPerSample for each component. Even in
/// this case, the writer must write all three values.
///
/// ### code_id
///
/// 258
///
/// ### datatype
///
/// - [`Short`]
///
/// ### count
///
/// N (= [`SamplesPerPixel`])
///
/// ### default_value
///
/// [1]
///
/// [`Short`]: ifd.Datatype.html
/// [`SamplesPerPixel`]: tag.SamplesPerPixel.html
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BitsPerSample {}

impl Tag for BitsPerSample {
    type Value = val::BitsPerSample<DynamicTone>;
    
    const ID: u16 = 258;
    const DEFAULT_VALUE: Option<val::BitsPerSample<DynamicTone>> = Some(val::BitsPerSample::C1(DynamicTone::new(1)));
}

/// The tag for compression scheme used in the image.
///
/// This tag's associated value is treated as `Option<`[`Compression`]`>` in [`ImageHeader`].
/// In case of non-compression it is `None`.
///
/// There is multiple compression scheme but the following are supported.
///
/// - Supported
///     - 1 = No compression
///     - 5 = LZW
/// - Not yet supported
///     - Baseline
///         - 2 = CCITT modified Huffman RLE
///         - 32773 = PackBits compression, aka Macintoch RLE
///     - Extensions
///         - 3 = CCITT Group 3 fax encoding
///         - 4 = CCITT Group 4 fax encoding
///         - 6 = JPEG (old style)
///         - 7 = JPEG (new style)
///         - 8 = Zip
///
/// ### code_id
///
/// 259
///
/// ### datatype
///
/// - [`Short`]
///
/// ### count
///
/// 1
///
/// ### default_value
///
/// 1 (= No compression)
///
/// [`Short`]: ifd.DataType.html
/// [`Compression`]: image.Compression.html
/// [`ImageHeader`]: image.ImageHeader.html
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Compression {}

impl Tag for Compression {
    type Value = Option<val::Compression>;

    const ID: u16 = 259;
    const DEFAULT_VALUE: Option<Option<val::Compression>> = Some(None);
}

/// The tag for the color space of the image data.
///
/// This tag's associated value is treated as [`PhotometricInterpretation`] in [`ImageHeader`].
///
/// There is multiple color space scheme but the following are supported.
///
/// - Supported
///     - 0 = WhiteIsZero
///     - 1 = BlackIsZero
///     - 2 = RGB
///     - 3 = Palette
///     - 4 = TransparencyMask
///     - 5 = CMYK
///     - 6 = YCbCr
///
/// ### code_id
///
/// 262
///
/// ### datatype
///
/// - [`Short`]
///
/// ### count
///
/// 1
///
/// ### default_value
///
/// None
///
/// [`Short`]: ifd.Datatype.html
/// [`PhotometricInterpretation`]: image.PhotometricInterpretation.html
/// [`ImageHeader`]: image.ImageHeader.html
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PhotometricInterpretation {}

impl Tag for PhotometricInterpretation {
    type Value = val::PhotometricInterpretation;

    const ID: u16 = 262;
}

/// The tag for the byte offset of strip.
///
/// The offset is specified with respect to the beginning of the TIFF file.
/// This implies that each strip has a location independent of the locations of other strips.
///
/// ### code_id
///
/// 273
///
/// ### datatype
///
/// - [`Short`]
/// - [`Long`]
///
/// ### count
///
/// - `StripsPerImage` (if [`PlanarConfiguration`] is equal to 1.)
/// - `StripsPerImage` * [`SamplesPerPixel`] (if [`PlanarConfiguration`] is equal to 2.)
///
/// `StripsPerImage` = floor(([`ImageLength`] + [`RowsPerStrip`] - 1)/[`RowsPerStrip`])
///
/// ### default_value
///
/// None
///
/// [`Short`]: ifd.DataType.html
/// [`Long`]: ifd.DataType.html
/// [`PlanarConfiguration`]: tag.PlanarConfiguration.html
/// [`SamplesPerPixel`]: tag.SamplesPerPixel.html
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StripOffsets {}

impl Tag for StripOffsets {
    type Value = Values;

    const ID: u16 = 273;
}

/// The tag for the number of components of pixel.
///
/// `SamplesPerPixel` is usually 1 for bilevel, grayscale and palette images.
/// `SamplesPerPixel` is usually 3 for RGB.
/// `SamplesPerPixel` is usually 4 for CMYK.
/// If the value is higher, [`ExtraSamples`] should give an indication of the meaning
/// of the additional channels. (However, [`ExtraSamples`] is not yet supported.)
///
/// ### code_id
///
/// 277
///
/// ### datatype
///
/// - [`Short`]
///
/// ### count
///
/// 1
///
/// ### default_value
///
/// 1
///
/// [`Short`]: ifd.Datatype.html
/// [`ExtraSamples`]: tag.ExtraSamples.html
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SamplesPerPixel {}

impl Tag for SamplesPerPixel {
    type Value = Short;

    const ID: u16 = 277;
}

/// The tag for the number of rows per strip.
///
/// TIFF image data can be organized into strips for faster random access and efficient I/O buffering.
/// [`RowsPerStrip1] and [`ImageLength`] together tell us the number of strips in the entire image.
///
/// `StripsPerImage` = floor(([`ImageLength`] + [`RowsPerStrip`] - 1)/[`RowsPerStrip`])
///
/// `StripsPerImage` is used to calculate [`StripOffsets`] and [`StripByteCounts`] for the image.
///
/// ### code_id
///
/// 278
///
/// ### datatype
///
/// - [`Short`]
/// - [`Long`]
///
/// ### default_value
///
/// 2^32 - 1
///
/// [`Short`]: ifd.DataType.html
/// [`Long`]: ifd.DataType.html
/// [`RowsPerStrip`]: tag.RowsPerStrip.html
/// [`ImageLength`]: tag.ImageLength.html
/// [`StripOffsets`]: tag.StripOffsets.html
/// [`StripByteCounts`]: tag.StripByteCounts.html
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RowsPerStrip {}

impl Tag for RowsPerStrip {
    type Value = Value;

    const ID: u16 = 278;
}

/// The tag for the number of bytes in the strip after compression per strip.
///
/// ### code_id
///
/// 279
///
/// ### datatype
///
/// - [`Short`]
/// - [`Long`]
///
/// ### count
///
/// - `StripsPerImage` (if [`PlanarConfiguration`] is equal to 1.)
/// - `StripsPerImage` * [`SamplesPerPixel`] (if [`PlanarConfiguration`] is equal to 2.)
///
/// `StripsPerImage` = floor(([`ImageLength`] + [`RowsPerStrip`] - 1)/[`RowsPerStrip`])
///
/// ### default_value
///
/// None
///
/// [`Short`]: ifd.Datatype.html
/// [`Long`]: ifd.Datatype.html
/// [`PlanarConfiguration`]: tag.PlanarConfiguration.html
/// [`RowsPerStrip`]: tag.RowsPerStrip.html
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StripByteCounts {}

impl Tag for StripByteCounts {
    type Value = Values;

    const ID: u16 = 279;
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
