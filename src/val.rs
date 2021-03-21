use crate::{byte::{Endian, EndianRead, SeekExt}, num::{DynamicTone, T16, T8}};
use crate::data::{DataType, Entry};
use crate::decode::{Decoded, Decoder};
use crate::error::{DecodeError, DecodeResult, DecodingError};
use crate::{field_is_data_pointer, valid_count};
use crate::num::{Tone};
use std::convert::From;
use std::io::{self, Seek};
use std::ops::Deref;

pub type Byte = u8;
pub type Bytes = Vec<u8>;
pub type Short = u16;
pub type Shorts = Vec<u16>;
pub type Long = u32;
pub type Longs = Vec<u32>;

#[derive(Debug, Clone)]
pub enum Value {
    Short(Short),
    Long(Long),
}

impl Value {
    pub fn as_long(self) -> Long {
        match self {
            Value::Short(x) => x as Long,
            Value::Long(x) => x,
        }
    }

    pub fn as_size(self) -> usize {
        match self {
            Value::Short(x) => x as usize,
            Value::Long(x) => x as usize,
        }
    }
}

impl From<Short> for Value {
    fn from(x: Short) -> Self {
        Value::Short(x)
    }
}

impl From<Long> for Value {
    fn from(x: Long) -> Self {
        Value::Long(x)
    }
}

pub enum Values {
    Shorts(Shorts),
    Longs(Longs),
}

impl Values {
    pub fn as_long(self) -> Longs {
        match self {
            Values::Shorts(x) => x.into_iter().map(|x| x as Long).collect(),
            Values::Longs(x) => x,
        }
    }

    pub fn as_size(self) -> Vec<usize> {
        match self {
            Values::Shorts(x) => x.into_iter().map(|x| x as usize).collect(),
            Values::Longs(x) => x.into_iter().map(|x| x as usize).collect()
        }
    }
}

impl From<Shorts> for Values {
    fn from(x: Shorts) -> Self {
        Values::Shorts(x)
    }
}

impl From<Longs> for Values {
    fn from(x: Longs) -> Self {
        Values::Longs(x)
    }
}

pub struct Rational {
    numerator: u32,
    denominator: u32,
}

impl Rational {
    pub fn new(numerator: u32, denominator: u32) -> Self {
        Rational {
            numerator,
            denominator,
        }
    }
}

macro_rules! decodefrom_1 {
    ($name:ident, $datatype:pat, $method:path) => {
        impl Decoded for $name {
            fn decode<'a, R: io::Read + io::Seek>(
                reader: &'a mut R,
                endian: &'a Endian,
                entry: Entry,
            ) -> DecodeResult<$name> {
                valid_count!(entry, 1..2)?;

                match entry.ty() {
                    $datatype => {
                        let mut field = entry.field();
                        let val = $method(&mut field, endian)?;
                        Ok(val)
                    }
                    x => Err(DecodeError::from(DecodingError::InvalidDataType(x))),
                }
            }
        }
    };
}

macro_rules! decodefrom_n {
    ($name:ident, $datatype:pat, $method:path) => {
        impl Decoded for $name {
            fn decode<'a, R: io::Read + io::Seek>(
                reader: &'a mut R,
                endian: &'a Endian,
                entry: Entry,
            ) -> DecodeResult<$name> {
                valid_count!(entry, 1..)?;

                match entry.ty() {
                    $datatype => {
                        let count = entry.count();
                        let mut data = vec![];
                        if entry.overflow() {
                            let next = entry.field().read_u32(&endian)?;
                            reader.goto(next as u64)?;

                            for _ in 0..count {
                                let val = $method(reader, &endian)?;
                                data.push(val);
                            }
                        } else {
                            for _ in 0..count {
                                let mut field = entry.field();
                                let val = $method(&mut field, &endian)?;
                                data.push(val);
                            }
                        }

                        Ok(data)
                    }
                    x => Err(DecodeError::from(DecodingError::InvalidDataType(x))),
                }
            }
        }
    };
}

decodefrom_1!(Byte, DataType::Byte, EndianRead::read_u8);
decodefrom_1!(Short, DataType::Short, EndianRead::read_u16);
decodefrom_1!(Long, DataType::Long, EndianRead::read_u32);

decodefrom_n!(Bytes, DataType::Byte, EndianRead::read_u8);
decodefrom_n!(Shorts, DataType::Short, EndianRead::read_u16);
decodefrom_n!(Longs, DataType::Long, EndianRead::read_u32);

impl Decoded for Value {
    fn decode<'a, R: io::Read + io::Seek>(
        reader: &'a mut R,
        endian: &'a Endian,
        entry: Entry,
    ) -> DecodeResult<Self> {
        match entry.ty() {
            DataType::Short => {
                let val: u16 = Decoded::decode(reader, endian, entry)?;
                Ok(Value::Short(val))
            }
            DataType::Long => {
                let val: u32 = Decoded::decode(reader, endian, entry)?;
                Ok(Value::Long(val))
            }
            x => Err(DecodeError::from(DecodingError::InvalidDataType(x))),
        }
    }
}

impl Decoded for Values {
    fn decode<'a, R: io::Read + io::Seek>(
        reader: &'a mut R,
        endian: &'a Endian,
        entry: Entry,
    ) -> DecodeResult<Self> {
        match entry.ty() {
            DataType::Short => {
                let val: Shorts = Decoded::decode(reader, endian, entry)?;
                Ok(Values::Shorts(val))
            }
            DataType::Long => {
                let val: Longs = Decoded::decode(reader, endian, entry)?;
                Ok(Values::Longs(val))
            }
            x => Err(DecodeError::from(DecodingError::InvalidDataType(x))),
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
    fn decode<'a, R: io::Read + io::Seek>(
        reader: &'a mut R,
        endian: &'a Endian,
        entry: Entry,
    ) -> DecodeResult<Self> {
        valid_count!(entry, 1..2)?;

        match entry.ty() {
            DataType::Short => {
                let val = entry.field().read_u16(endian)?;
                match val {
                    0 => Ok(PhotometricInterpretation::WhiteIsZero),
                    1 => Ok(PhotometricInterpretation::BlackIsZero),
                    2 => Ok(PhotometricInterpretation::RGB),
                    3 => Ok(PhotometricInterpretation::Palette),
                    4 => Ok(PhotometricInterpretation::TransparencyMask),
                    5 => Ok(PhotometricInterpretation::CMYK),
                    6 => Ok(PhotometricInterpretation::YCbCr),
                    7 => Ok(PhotometricInterpretation::CIELab),
                    n => Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n]))),
                }
            }
            x => Err(DecodeError::from(DecodingError::InvalidDataType(x))),
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
    fn decode<'a, R: io::Read + io::Seek>(
        reader: &'a mut R,
        endian: &'a Endian,
        entry: Entry,
    ) -> DecodeResult<Self> {
        valid_count!(entry, 1..2)?;
        let val = entry.field().read_u16(endian)?;
        match val {
            1 => Ok(None),
            5 => Ok(Some(Compression::LZW)),
            n => Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n]))),
        }
    }
}

/// Bits/Sample
///
/// IFD constructs this with `tag::BitsPerSample`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BitsPerSample<T: Tone> {
    C1(T),
    C3(T),
    C4(T),
}

impl<T: Tone> BitsPerSample<T> {
    pub fn len(&self) -> usize {
        match self {
            BitsPerSample::C1(_) => 1,
            BitsPerSample::C3(_) => 3,
            BitsPerSample::C4(_) => 4,
        }
    }

    pub fn tone(&self) -> &T {
        match self {
            BitsPerSample::C1(x) => x,
            BitsPerSample::C3(x) => x,
            BitsPerSample::C4(x) => x,
        }
    }
}

impl Decoded for BitsPerSample<DynamicTone> {
    fn decode<'a, R: io::Read + io::Seek>(
        reader: &'a mut R,
        endian: &'a Endian,
        entry: Entry,
    ) -> DecodeResult<Self> {
        valid_count!(entry, vec![1, 3, 4])?;

        if field_is_data_pointer!(reader, endian, entry) {
            // count = 3 or 4
            match entry.count() {
                3 => {
                    let val1 = reader.read_u16(&endian)?;
                    let val2 = reader.read_u16(&endian)?;
                    let val3 = reader.read_u16(&endian)?;

                    if val1 != val2 || val1 != val3 {
                        return Err(DecodeError::from(DecodingError::UnsupportedValue(vec![val1, val2, val3])))
                    }

                    let tone = match val1 {
                        n if n == 8 || n == 16 => DynamicTone::new(n as usize),
                        n => return Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n])))
                    };

                    Ok(BitsPerSample::C3(tone))
                }
                4 => {
                    let val1 = reader.read_u16(&endian)?;
                    let val2 = reader.read_u16(&endian)?;
                    let val3 = reader.read_u16(&endian)?;
                    let val4 = reader.read_u16(&endian)?;

                    if val1 != val2 || val1 != val3 || val1 != val4 {
                        return Err(DecodeError::from(DecodingError::UnsupportedValue(vec![val1, val2, val3, val4])))
                    }

                    let tone = match val1 {
                        n if n == 8 || n == 16 => DynamicTone::new(n as usize),
                        n => return Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n])))
                    };

                    Ok(BitsPerSample::C4(tone))
                }
                n => Err(DecodeError::from(DecodingError::InvalidCount(n))),
            }
        } else {
            // count = 1
            let val1 = entry.field().read_u16(&endian)?;

            let tone = match val1 {
                n if n == 8 || n == 16 => DynamicTone::new(n as usize),
                n => return Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n])))
            };

            Ok(BitsPerSample::C1(tone))
        }
    }
}
