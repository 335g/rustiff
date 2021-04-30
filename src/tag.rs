
use crate::decode::Decoded;
use crate::encode::Encoded;
use crate::val;

pub trait Tag {
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