use std::ops::{Deref, RangeFrom};

use crate::{decode::Decoded, element::AnyElement, encode::Encoded, error::DecodingError};

#[derive(Debug, Clone)]
pub enum Value {
    Short(u16),
    Long(u32),
}

impl Value {
    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn as_long(self) -> u32 {
        match self {
            Value::Short(x) => x as u32,
            Value::Long(x) => x,
        }
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn as_size(self) -> usize {
        match self {
            Value::Short(x) => x as usize,
            Value::Long(x) => x as usize,
        }
    }
}

impl From<u16> for Value {
    fn from(x: u16) -> Self {
        Value::Short(x)
    }
}

impl From<u32> for Value {
    fn from(x: u32) -> Self {
        Value::Long(x)
    }
}

#[derive(Debug, Clone)]
pub struct ImageWidth(Value);

impl ImageWidth {
    pub fn as_size(self) -> usize {
        match self.0 {
            Value::Short(x) => x as usize,
            Value::Long(x) => x as usize,
        }
    }
}

impl Decoded for ImageWidth {
    type Element = Value;
    type Poss = usize;

    const POSSIBLE_COUNT: Self::Poss = 1;

    #[inline]
    fn decoded(mut elements: Vec<Value>) -> Result<Self, DecodingError> {
        Ok(Self(elements.remove(0)))
    }
}

impl Encoded<ImageWidth> for Value {
    #[inline(always)]
    fn encoded(val: ImageWidth) -> Self {
        val.0
    }
}

#[derive(Debug, Clone)]
pub struct ImageLength(Value);

impl ImageLength {
    pub fn as_size(self) -> usize {
        match self.0 {
            Value::Short(x) => x as usize,
            Value::Long(x) => x as usize,
        }
    }
}

impl Decoded for ImageLength {
    type Element = Value;
    type Poss = usize;

    const POSSIBLE_COUNT: Self::Poss = 1;

    #[inline]
    fn decoded(mut elements: Vec<Value>) -> Result<Self, DecodingError> {
        Ok(Self(elements.remove(0)))
    }
}

impl Encoded<ImageLength> for Value {
    #[inline(always)]
    fn encoded(val: ImageLength) -> Self {
        val.0
    }
}

/// Bits/Sample
///
/// IFD constructs this with `tag::BitsPerSample`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BitsPerSample {
    C1([u16; 1]),
    C3([u16; 3]),
    C4([u16; 4]),
}

impl BitsPerSample {
    #[allow(missing_docs)]
    pub fn len(&self) -> usize {
        match self {
            BitsPerSample::C1(_) => 1,
            BitsPerSample::C3(_) => 3,
            BitsPerSample::C4(_) => 4,
        }
    }
}

impl Decoded for BitsPerSample {
    type Element = u16;
    type Poss = [usize; 3];

    const POSSIBLE_COUNT: Self::Poss = [1, 3, 4];

    fn decoded(elements: Vec<Self::Element>) -> Result<Self, DecodingError> {
        match elements.len() {
            1 => {
                let a = elements[0];
                if a == 0 {
                    let ty = std::any::type_name::<Self>();
                    Err(DecodingError::InvalidValue(AnyElement::U16(0)))
                } else {
                    Ok(BitsPerSample::C1([a]))
                }
            }
            3 => {
                let a = elements[0];
                let b = elements[1];
                let c = elements[2];

                if a == 0 || b == 0 || c == 0 {
                    let ty = std::any::type_name::<Self>();
                    Err(DecodingError::InvalidValue(AnyElement::U16(0)))
                } else {
                    Ok(BitsPerSample::C3([a, b, c]))
                }
            }
            4 => {
                let a = elements[0];
                let b = elements[1];
                let c = elements[2];
                let d = elements[3];

                if a == 0 || b == 0 || c == 0 || d == 0 {
                    let ty = std::any::type_name::<Self>();

                    Err(DecodingError::InvalidValue(AnyElement::U16(0)))
                } else {
                    Ok(BitsPerSample::C4([a, b, c, d]))
                }
            }
            _ => unreachable!(), // Possible limits the numbers to 1, 3, and 4.
        }
    }
}

impl Encoded<BitsPerSample> for Vec<u16> {
    fn encoded(val: BitsPerSample) -> Self {
        match val {
            BitsPerSample::C1(t) => t.into(),
            BitsPerSample::C3(t) => t.into(),
            BitsPerSample::C4(t) => t.into(),
        }
    }
}

/// Compression scheme used on the image data.
///
/// IFD constructs this with `tag::Compression`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Compression {
    /// LZW compression
    LZW,
}

impl Decoded for Option<Compression> {
    type Element = u16;
    type Poss = usize;

    const POSSIBLE_COUNT: Self::Poss = 1;

    fn decoded(mut elements: Vec<u16>) -> Result<Self, DecodingError> {
        let val = elements.remove(0);

        match val {
            1 => Ok(None),
            5 => Ok(Some(Compression::LZW)),
            n => Err(DecodingError::InvalidValue(AnyElement::U16(n))),
        }
    }
}

impl Encoded<Option<Compression>> for u16 {
    fn encoded(val: Option<Compression>) -> Self {
        match val {
            None => 1,
            Some(Compression::LZW) => 5,
        }
    }
}

/// The color space of the image data.
///
/// IFD constructs this with `tag::PhotometricInterpretation`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PhotometricInterpretation {
    /// For bilievel and grayscale images.
    ///
    /// 0 is imaged as white. 2**BitsPerSample - 1 is imaged as black.
    /// If GrayResponseCurve exists, it overrides the PhotometricInterpretation
    /// value.
    ///
    /// TODO: impl tag::GrayResponseCurve
    WhiteIsZero,

    /// For bilievel and grayscale images.
    ///
    /// 0 is imaged as black. 2**BitsPerSample - 1 is imaged as white.
    /// If GrayResponseCurve exists, it overrides the PhotometricInterpretation
    /// value.
    ///
    /// TODO: impl tag::GrayResponseCurve
    BlackIsZero,

    /// Rgb color space.
    ///
    /// In this mode, a color is described as a combination of three primary
    /// colors of light (red, green, blue) in particular concentrations. For each
    /// of the three samples, 0 represents minimum intensity, and  2**BitsPerSample - 1
    /// represents maximum intensity.
    RGB,

    /// Rgb color space with lookup table.
    ///
    /// In this mode, a color is described single component. The value of component
    /// is used as an index into ColorMap (often called `Lookup table`). The value of
    /// component will be converted to a RGB triplet defining actual color by Lookup table.
    ///
    /// #need
    ///
    /// - IFD must have `tag::ColorMap` tag,
    /// - IFD must have `tag::SamplesPerPixel` with value 1.
    Palette,

    /// Irregularly shaped region
    ///
    /// This means that the image is used to define an irregularly shaped region of
    /// another image in the same TIFF file. Packbits compression is recommended.
    /// The 1-bits define the interior of the region. The 0-bits define the exterior
    /// of the region.
    ///
    /// #need
    ///
    /// - IFD must have `tag::SamplesPerPixel` with value 1.
    /// - IFD must have `tag::BitsPerSample` with value 1.
    ///
    /// #must
    ///
    /// - `ImageLength` must be the same as `ImageLength`.
    TransparencyMask,

    /// Cmyk color space.
    ///
    /// (0,0,0,0) represents white.
    CMYK,

    #[allow(missing_docs)]
    YCbCr,

    #[allow(missing_docs)]
    CIELab,
}

impl Decoded for PhotometricInterpretation {
    type Element = u16;
    type Poss = usize;

    const POSSIBLE_COUNT: Self::Poss = 1;

    fn decoded(elements: Vec<u16>) -> Result<Self, DecodingError> {
        match elements[0] {
            0 => Ok(PhotometricInterpretation::WhiteIsZero),
            1 => Ok(PhotometricInterpretation::BlackIsZero),
            2 => Ok(PhotometricInterpretation::RGB),
            3 => Ok(PhotometricInterpretation::Palette),
            4 => Ok(PhotometricInterpretation::TransparencyMask),
            5 => Ok(PhotometricInterpretation::CMYK),
            6 => Ok(PhotometricInterpretation::YCbCr),
            7 => Ok(PhotometricInterpretation::CIELab),
            n => Err(DecodingError::InvalidValue(AnyElement::U16(n))),
        }
    }
}

impl Encoded<PhotometricInterpretation> for u16 {
    fn encoded(val: PhotometricInterpretation) -> Self {
        match val {
            PhotometricInterpretation::WhiteIsZero => 0,
            PhotometricInterpretation::BlackIsZero => 1,
            PhotometricInterpretation::RGB => 2,
            PhotometricInterpretation::Palette => 3,
            PhotometricInterpretation::TransparencyMask => 4,
            PhotometricInterpretation::CMYK => 5,
            PhotometricInterpretation::YCbCr => 6,
            PhotometricInterpretation::CIELab => 7,
        }
    }
}

///
#[derive(Debug)]
pub struct StripOffsets(Vec<Value>);

impl Deref for StripOffsets {
    type Target = Vec<Value>;

    fn deref(&self) -> &Vec<Value> {
        &self.0
    }
}

impl Decoded for StripOffsets {
    type Element = Value;
    type Poss = RangeFrom<usize>;

    const POSSIBLE_COUNT: Self::Poss = 1..;

    #[inline]
    fn decoded(elements: Vec<Value>) -> Result<Self, DecodingError> {
        Ok(StripOffsets(elements))
    }
}

impl Encoded<StripOffsets> for Vec<Value> {
    #[inline(always)]
    fn encoded(val: StripOffsets) -> Self {
        val.0
    }
}

///
#[derive(Debug)]
pub struct SamplesPerPixel(u16);

impl Deref for SamplesPerPixel {
    type Target = u16;

    fn deref(&self) -> &u16 {
        &self.0
    }
}

impl Decoded for SamplesPerPixel {
    type Element = u16;
    type Poss = usize;

    const POSSIBLE_COUNT: Self::Poss = 1;

    fn decoded(mut elements: Vec<u16>) -> Result<Self, DecodingError> {
        let val = elements.remove(0);

        // TODO: really 1,3,4 ?
        match val {
            1 | 3 | 4 => Ok(SamplesPerPixel(val)),
            n => Err(DecodingError::InvalidValue(AnyElement::U16(n))),
        }
    }
}

impl Encoded<SamplesPerPixel> for u16 {
    #[inline(always)]
    fn encoded(val: SamplesPerPixel) -> Self {
        val.0
    }
}

///
#[derive(Debug)]
pub struct RowsPerStrip(Value);

impl RowsPerStrip {
    pub fn as_size(self) -> usize {
        match self.0 {
            Value::Short(x) => x as usize,
            Value::Long(x) => x as usize,
        }
    }
}

impl RowsPerStrip {
    pub const fn default_value() -> Self {
        RowsPerStrip(Value::Long(u32::MAX))
    }
}

impl Decoded for RowsPerStrip {
    type Element = Value;
    type Poss = usize;

    const POSSIBLE_COUNT: Self::Poss = 1;

    #[inline]
    fn decoded(mut elements: Vec<Self::Element>) -> Result<Self, DecodingError> {
        Ok(RowsPerStrip(elements.remove(0)))
    }
}

impl Encoded<RowsPerStrip> for Value {
    #[inline(always)]
    fn encoded(val: RowsPerStrip) -> Self {
        val.0
    }
}

#[derive(Debug)]
pub struct StripByteCounts(Vec<Value>);

impl Deref for StripByteCounts {
    type Target = Vec<Value>;

    fn deref(&self) -> &Vec<Value> {
        &self.0
    }
}

impl Decoded for StripByteCounts {
    type Element = Value;
    type Poss = RangeFrom<usize>;

    const POSSIBLE_COUNT: Self::Poss = 1..;

    #[inline]
    fn decoded(elements: Vec<Value>) -> Result<Self, DecodingError> {
        Ok(StripByteCounts(elements))
    }
}

impl Encoded<StripByteCounts> for Vec<Value> {
    #[inline(always)]
    fn encoded(val: StripByteCounts) -> Self {
        val.0
    }
}

/// Difference from the previous pixel
///
/// IFD constructs this with `tag::Predictor`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Predictor {
    None,
    Horizontal,
}

impl Decoded for Predictor {
    type Element = u16;
    type Poss = usize;

    const POSSIBLE_COUNT: Self::Poss = 1;

    fn decoded(mut elements: Vec<Self::Element>) -> Result<Self, DecodingError> {
        let val = elements.remove(0);

        match val {
            1 => Ok(Predictor::None),
            2 => Ok(Predictor::Horizontal),
            n => Err(DecodingError::InvalidValue(AnyElement::U16(n))),
        }
    }
}

impl Encoded<Predictor> for u16 {
    fn encoded(val: Predictor) -> Self {
        match val {
            Predictor::None => 1,
            Predictor::Horizontal => 2,
        }
    }
}
