use crate::byte::{Endian, EndianRead, SeekExt};
use crate::data::{DataType, Entry};
use crate::decode::{Decoded, Decoder};
use crate::error::{DecodeError, DecodeResult, DecodeValueError};
use crate::{field_is_data_pointer, valid_count};
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
    pub fn as_long(&self) -> Long {
        match *self {
            Value::Short(x) => x as Long,
            Value::Long(x) => x
        }
    }

    pub fn as_size(&self) -> usize {
        match *self {
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
        Rational { numerator, denominator }
    }
}

macro_rules! decodefrom_1 {
    ($name:ident, $datatype:pat, $method:path) => {
        impl Decoded for $name {
            fn decode<R: io::Read + io::Seek>(
                decoder: &mut Decoder<R>,
                entry: &Entry,
            ) -> DecodeResult<$name> {
                valid_count!(entry, 1..2)?;
                let endian = decoder.endian();

                match entry.ty() {
                    $datatype => {
                        let mut field = entry.field();
                        let val = $method(&mut field, &endian)?;
                        Ok(val)
                    }
                    x => Err(DecodeError::from(DecodeValueError::InvalidDataType(x))),
                }
            }
        }
    };
}

macro_rules! decodefrom_n {
    ($name:ident, $datatype:pat, $method:path) => {
        impl Decoded for $name {
            fn decode<R: io::Read + io::Seek>(
                decoder: &mut Decoder<R>,
                entry: &Entry,
            ) -> DecodeResult<$name> {
                valid_count!(entry, 1..)?;
                let endian = decoder.endian().clone();

                match entry.ty() {
                    $datatype => {
                        let count = entry.count();
                        let mut data = vec![];
                        if entry.overflow() {
                            let reader = decoder.reader();
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
                    x => Err(DecodeError::from(DecodeValueError::InvalidDataType(x))),
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
    fn decode<R: io::Read + io::Seek>(
        decoder: &mut Decoder<R>,
        entry: &Entry,
    ) -> DecodeResult<Self> {
        match entry.ty() {
            DataType::Short => {
                let val: u16 = Decoded::decode(decoder, entry)?;
                Ok(Value::Short(val))
            }
            DataType::Long => {
                let val: u32 = Decoded::decode(decoder, entry)?;
                Ok(Value::Long(val))
            }
            x => Err(DecodeError::from(DecodeValueError::InvalidDataType(x))),
        }
    }
}

impl Decoded for Values {
    fn decode<R: io::Read + io::Seek>(
        decoder: &mut Decoder<R>,
        entry: &Entry,
    ) -> DecodeResult<Self> {
        match entry.ty() {
            DataType::Short => {
                let val: Shorts = Decoded::decode(decoder, entry)?;
                Ok(Values::Shorts(val))
            }
            DataType::Long => {
                let val: Longs = Decoded::decode(decoder, entry)?;
                Ok(Values::Longs(val))
            }
            x => Err(DecodeError::from(DecodeValueError::InvalidDataType(x))),
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
    fn decode<R: io::Read + io::Seek>(
        decoder: &mut Decoder<R>,
        entry: &Entry,
    ) -> DecodeResult<Self> {
        valid_count!(entry, 1..2)?;
        let endian = decoder.endian();
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
                    n => Err(DecodeError::from(DecodeValueError::InvalidValue(vec![
                        n as u32,
                    ]))),
                }
            }
            x => Err(DecodeError::from(DecodeValueError::InvalidDataType(x))),
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
    fn decode<R: io::Read + io::Seek>(
        decoder: &mut Decoder<R>,
        entry: &Entry,
    ) -> DecodeResult<Self> {
        valid_count!(entry, 1..2)?;
        let val = entry.field().read_u16(decoder.endian())?;
        match val {
            1 => Ok(None),
            5 => Ok(Some(Compression::LZW)),
            n => Err(DecodeError::from(DecodeValueError::InvalidValue(vec![
                n as u32,
            ]))),
        }
    }
}

/// Bits/Sample
///
/// IFD constructs this with `tag::BitsPerSample`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BitsPerSample {
    C1(u16),
    C3(u16, u16, u16),
    C4(u16, u16, u16, u16),
}

impl BitsPerSample {
    pub fn len(&self) -> usize {
        match *self {
            BitsPerSample::C1(_) => 1,
            BitsPerSample::C3(_, _, _) => 3,
            BitsPerSample::C4(_, _, _, _) => 4,
        }
    }

    pub fn max(&self) -> u16 {
        use std::cmp::max;

        match *self {
            BitsPerSample::C1(v1) => v1,
            BitsPerSample::C3(v1, v2, v3) => max(max(v1, v2), v3),
            BitsPerSample::C4(v1, v2, v3, v4) => max(max(max(v1, v2), v3), v4),
        }
    }
}

impl Decoded for BitsPerSample {
    fn decode<R: io::Read + io::Seek>(
        decoder: &mut Decoder<R>,
        entry: &Entry,
    ) -> DecodeResult<Self> {
        valid_count!(entry, vec![1, 3, 4])?;
        let endian = decoder.endian().clone();
        let reader = decoder.reader();

        if field_is_data_pointer!(reader, &endian, entry) {
            // count = 3 or 4
            match entry.count() {
                3 => {
                    let val1 = reader.read_u16(&endian)?;
                    let val2 = reader.read_u16(&endian)?;
                    let val3 = reader.read_u16(&endian)?;

                    Ok(BitsPerSample::C3(val1, val2, val3))
                }
                4 => {
                    let val1 = reader.read_u16(&endian)?;
                    let val2 = reader.read_u16(&endian)?;
                    let val3 = reader.read_u16(&endian)?;
                    let val4 = reader.read_u16(&endian)?;

                    Ok(BitsPerSample::C4(val1, val2, val3, val4))
                }
                n => unreachable!("Unreachable by invalid data count: {}", n),
            }
        } else {
            // count = 1
            let val = entry.field().read_u16(&endian)?;
            Ok(BitsPerSample::C1(val))
        }
    }
}
