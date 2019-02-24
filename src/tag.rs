
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

/// Trait for tag.
pub trait TagType: Fail + Clone + Copy {
    /// Decoded type by this tag.
    type Value: fmt::Debug + Send + Sync;

    /// Identifier.
    ///
    /// This must not be equal to supported tag's identifier.
    /// If both identifier are equal, error (= `ImpossibleTag`) occurs when you use this tag.
    fn id(&self) -> u16;

    /// Default value when `ifd::IFD` doesn't have the value with this tag.
    fn default_value() -> Option<Self::Value>;

    /// Decode method.
    fn decode<'a, R: Read + Seek + 'a>(&'a self, reader: R, offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> Result<Self::Value, DecodeError>;
}

/// Error for impossible tag.
#[derive(Debug, Clone)]
pub struct ImpossibleTag<T: TagType>(T);

impl<T> ImpossibleTag<T> where T: TagType {
    /// Move detail tag.
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

        /// Tag to get associated value from `ifd::IFD`.
        ///
        /// A tag that conforms to `tag::TagType` changes this automatically.
        #[derive(Debug, Clone, Eq, PartialEq, Hash, Fail)]
        pub enum AnyTag {
            $(#[allow(missing_docs)]
            #[fail(display = "Supported tag: {{ name: $name, id: $id }}")]
            $name,)*

            /// Unsupported tag.
            ///
            /// `rustiff` user can use unsupported tag to implement `tag::TagType`.
            /// This tag chnages to `AnyTag::Custom` with `tag::TagType::id()`.
            #[fail(display = "Unsupported tag: {{ id: {} }}", _0)]
            Custom(u16),
        }

        impl AnyTag {
            /// Identifier
            ///
            /// Same as the value obtained by `tag::TagType::id`.
            pub fn id(&self) -> u16 {
                match *self {
                    $(AnyTag::$name => $id,)*
                    AnyTag::Custom(n) => n,
                }
            }

            /// Constructor
            pub(crate) fn from_u16(n: u16) -> AnyTag {
                match n {
                    $($id => AnyTag::$name,)*
                    _ => AnyTag::Custom(n),
                }
            }

        }

        impl<T> PartialEq<T> for AnyTag where T: TagType {
            fn eq(&self, rhs: &T) -> bool {
                match *self {
                    $(AnyTag::$name => TypeId::of::<$name>() == TypeId::of::<T>(),)*
                    AnyTag::Custom(n) => n == rhs.id(),
                }
            }
        }
    }
}

impl AnyTag {
    /// Check and construct
    ///
    /// Identifier of trying to use tag
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
    ($name:ident, $def:expr, $id:expr) => {
        impl TagType for $name {
            type Value = u32;
            
            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<u32> { $def }
            fn decode<'a, R: Read + Seek + 'a>(&'a self, mut _reader: R, mut offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> Result<Self::Value, DecodeError> {
                match datatype {
                    DataType::Short if count == 1 => Ok(offset.read_u16(endian)? as u32),
                    DataType::Long if count == 1 => Ok(offset.read_u32(endian)?),
                    _ => {
                        let anytag = AnyTag::try_from(*self)?;
                        
                        Err(DecodeError::from(DecodeErrorKind::UnsupportedDataTypeAndCount { tag: anytag, datatype: datatype, count: count }))
                    },
                }
            }
        }
    }
}

macro_rules! short_value {
    ($name:ident, $def:expr, $id:expr) => {
        impl TagType for $name {
            type Value = u16;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<u16> { $def }
            fn decode<'a, R: Read + Seek + 'a>(&'a self, mut _reader: R, mut offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> Result<Self::Value, DecodeError> {
                match datatype {
                    DataType::Short if count == 1 => Ok(offset.read_u16(endian)?),
                    _ => {
                        let anytag = AnyTag::try_from(*self)?;
                        
                        Err(DecodeError::from(DecodeErrorKind::UnsupportedDataTypeAndCount { tag: anytag, datatype: datatype, count: count }))
                    },
                }
            }
        }
    }
}

macro_rules! short_or_long_values {
    ($name:ident, $def:expr, $id:expr) => {
        impl TagType for $name {
            type Value = Vec<u32>;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<Vec<u32>> { $def }
            fn decode<'a, R: Read + Seek + 'a>(&'a self, mut reader: R, mut offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> Result<Self::Value, DecodeError> {
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
                    _ => {
                        let anytag = AnyTag::try_from(*self)?;
                        
                        Err(DecodeError::from(DecodeErrorKind::UnsupportedDataTypeAndCount { tag: anytag, datatype: datatype, count: count }))
                    },
                }
            }
        }
    }
}

macro_rules! short_values {
    ($name:ident, $def:expr, $id:expr) => {
        impl TagType for $name {
            type Value = Vec<u16>;

            fn id(&self) -> u16 { $id }
            fn default_value() -> Option<Vec<u16>> { $def }
            fn decode<'a, R: Read + Seek + 'a>(&'a self, mut reader: R, mut offset: &'a [u8], endian: Endian, datatype: DataType, count: usize) -> Result<Self::Value, DecodeError> {
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
                    _ => {
                        let anytag = AnyTag::try_from(*self)?;

                        Err(DecodeError::from(DecodeErrorKind::UnsupportedDataTypeAndCount { tag: anytag, datatype: datatype, count: count }))
                    },
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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct ImageWidth;
short_or_long_value!(ImageWidth, None, 256);

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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct ImageLength;
short_or_long_value!(ImageLength, None, 257);

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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct BitsPerSample;
short_values!(BitsPerSample, Some(vec![1]), 258);

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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct Compression;
short_value!(Compression, Some(1), 259);

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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct PhotometricInterpretation;
short_value!(PhotometricInterpretation, None, 262);

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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct StripOffsets;
short_or_long_values!(StripOffsets, None, 273);

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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct SamplesPerPixel;
short_value!(SamplesPerPixel, Some(1), 277);

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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct RowsPerStrip;
short_or_long_value!(RowsPerStrip, Some(u32::max_value()), 278);

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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Fail)]
pub struct StripByteCounts;
short_or_long_values!(StripByteCounts, None, 279);

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

