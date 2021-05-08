use crate::{Data, byte::{Endian, EndianRead, SeekExt}, num::{DynamicTone, T16, T8}};
use crate::data::{DataType, Entry};
use crate::decode::{Element, Decoder, Codable, Decodable};
use crate::error::{DecodeError, DecodeResult, DecodingError};
use crate::{field_is_data_pointer, valid_count};
use crate::num::{Tone};
use std::{any::type_name, convert::From};
use std::io::{self, Seek};
use std::ops::Deref;
use std::ops::RangeFrom;

pub type Bytes = Vec<u8>;
pub type Shorts = Vec<u16>;
pub type Longs = Vec<u32>;

#[derive(Debug, Clone)]
pub enum Value {
    Short(u16),
    Long(u32),
}

impl Value {
    pub fn as_long(self) -> u32 {
        match self {
            Value::Short(x) => x as u32,
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

pub enum Values {
    Shorts(Vec<u16>),
    Longs(Vec<u32>),
}

impl Values {
    #[inline]
    pub fn as_long(self) -> Vec<u32> {
        self.map(|x| x as u32, |x| x)
    }

    #[inline]
    pub fn as_size(self) -> Vec<usize> {
        self.map(|x| x as usize, |x| x as usize)
    }

    pub fn map<F1, F2, T>(self, shorts_fn: F1, longs_fn: F2) -> Vec<T>
    where
        F1: Fn(u16) -> T,
        F2: Fn(u32) -> T,
    {
        match self {
            Values::Shorts(x) => x.into_iter().map(shorts_fn).collect(),
            Values::Longs(x) => x.into_iter().map(longs_fn).collect()
        }
    }
}

impl From<Vec<u16>> for Values {
    fn from(x: Vec<u16>) -> Self {
        Values::Shorts(x)
    }
}

impl From<Vec<u32>> for Values {
    fn from(x: Vec<u32>) -> Self {
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

// macro_rules! decodefrom_1 {
//     ($name:ident, $datatype:ident, $method:path) => {
//         impl Decoded for $name {
//             fn decode<'a, 'b, 'c, R>(reader: &'a mut R, endian: &'b Endian, entry: &'c Entry) -> Result<$name, DecodeError> 
//                 where 
//                     R: io::Read + io::Seek,
//                     'a: 'b,
//                     'a: 'c
//             {
//                 valid_count!(entry, 1..2, std::any::type_name::<Self>())?;

//                 match entry.ty() {
//                     $datatype => {
//                         let mut field = entry.field();
//                         let val = $method(&mut field, endian)?;
//                         Ok(val)
//                     }
//                     x => Err(DecodeError::from(DecodingError::InvalidDataType(x))),
//                 }
//             }
//         }
//         impl Codable for $name {
//             type Element = $name;
//         }

//         impl Decodable for $name {
//             type PossibleDataType = DataType;
//             type PossibleCount = usize;

//             const POSSIBLE_ELEMENT: PossibleElement<DataType, usize> = PossibleElement::new($datatype, 1);

//             fn decode(element: $name) -> Result<$name, DecodingError> {
//                 Ok(element)
//             }
//         }
//     };
// }

// macro_rules! decodefrom_n {
//     ($name:ident, $datatype:pat, $method:path) => {
//         impl Decoded for $name {
//             fn decode<'a, 'b, 'c, R>(reader: &'a mut R, endian: &'b Endian, entry: &'c Entry) -> Result<$name, DecodeError> 
//                 where 
//                     R: io::Read + io::Seek,
//                     'a: 'b,
//                     'a: 'c
//             {
//                 valid_count!(entry, 1.., std::any::type_name::<Self>())?;

//                 match entry.ty() {
//                     $datatype => {
//                         let count = entry.count();
//                         let mut data = vec![];
//                         if entry.overflow() {
//                             let next = entry.field().read_u32(&endian)?;
//                             reader.goto(next as u64)?;

//                             for _ in 0..count {
//                                 let val = $method(reader, &endian)?;
//                                 data.push(val);
//                             }
//                         } else {
//                             for _ in 0..count {
//                                 let mut field = entry.field();
//                                 let val = $method(&mut field, &endian)?;
//                                 data.push(val);
//                             }
//                         }

//                         Ok(data)
//                     }
//                     x => Err(DecodeError::from(DecodingError::InvalidDataType(x))),
//                 }
//             }
//         }

//         impl Codable for $name {
//             type Element = $name;
//         }

//         impl Decodable for $name {
//             type PossibleDataType = DataType;
//             type PossibleCount = RangeFrom<usize>;

//             const POSSIBLE_ELEMENT: PossibleElement<DataType, RangeFrom<usize>> = PossibleElement::new($datatype, 1..);

//             fn decode(element: $name) -> Result<$name, DecodingError> {
//                 Ok(element)
//             }
//         }
//     };
// }

impl Element for u8 {
    fn decode<R: io::Read + Seek>(reader: R, entry: Entry, endian: &Endian) -> Result<Self, DecodingError> {
        let count = entry.count();

        if count != 1 {
            return Err(DecodingError::InvalidCount(vec![(count, std::any::type_name::<u8>())]))
        }

        let ty = entry.ty();

        if ty != DataType::Byte {
            return Err(DecodingError::InvalidDataType(vec![(ty, std::any::type_name::<u8>())]))
        }

        let val = reader.read_u8(endian);

        todo!()
    }
}

impl Codable for u8 {
    type Element = u8;
}

impl Decodable for u8 {
    fn decode(val: u8) -> Result<Self, DecodingError> {
        Ok(val)
    }
}

impl Codable for u16 {
    type Element = u16;
}

impl Decodable for u16 {
    fn decode(val: u16) -> Result<Self, DecodingError> {
        Ok(val)
    }
}

impl Codable for u32 {
    type Element = u32;
}

impl Decodable for u32 {
    fn decode(val: u32) -> Result<Self, DecodingError> {
        Ok(val)
    }
}

// decodefrom_1!(u8, DataType::Byte, EndianRead::read_u8);
// decodefrom_1!(u16, DataType::Short, EndianRead::read_u16);
// decodefrom_1!(u32, DataType::Long, EndianRead::read_u32);

impl Codable for Vec<u8> {
    type Element = Vec<u8>;
}

impl Decodable for Vec<u8> {
    fn decode(val: Vec<u8>) -> Result<Self, DecodingError> {
        Ok(val)
    }
}

impl Codable for Vec<u16> {
    type Element = Vec<u16>;
}

impl Decodable for Vec<u16> {
    fn decode(val: Vec<u16>) -> Result<Self, DecodingError> {
        Ok(val)
    }
}

impl Codable for Vec<u32> {
    type Element = Vec<u32>;
}

impl Decodable for Vec<u32> {
    fn decode(val: Vec<u32>) -> Result<Self, DecodingError> {
        Ok(val)
    }
}

// decodefrom_n!(Bytes, DataType::Byte, EndianRead::read_u8);
// decodefrom_n!(Shorts, DataType::Short, EndianRead::read_u16);
// decodefrom_n!(Longs, DataType::Long, EndianRead::read_u32);

// impl Decoded for Value {
//     fn decode<'a, 'b, 'c, R>(reader: &'a mut R, endian: &'b Endian, entry: &'c Entry) -> Result<Self, DecodeError> 
//         where
//             R: io::Read + io::Seek,
//             'a: 'b,
//             'a: 'c
//     {
//         match entry.ty() {
//             DataType::Short => {
//                 let val: u16 = Decoded::decode(reader, endian, entry)?;
//                 Ok(Value::Short(val))
//             }
//             DataType::Long => {
//                 let val: u32 = Decoded::decode(reader, endian, entry)?;
//                 Ok(Value::Long(val))
//             }
//             x => Err(DecodeError::from(DecodingError::InvalidDataType(x))),
//         }
//     }
// }

impl Codable for Value {
    type Element = Value;
}

impl Decodable for Value {
    fn decode(val: Value) -> Result<Self, DecodingError> {
        Ok(val)
    }
}

// impl Decoded for Values {
//     fn decode<'a, 'b, 'c, R>(reader: &'a mut R, endian: &'b Endian, entry: &'c Entry) -> Result<Self, DecodeError> 
//         where
//             R: io::Read + io::Seek,
//             'a: 'b,
//             'a: 'c
//     {
//         match entry.ty() {
//             DataType::Short => {
//                 let val: Vec<u16> = Decoded::decode(reader, endian, entry)?;
//                 Ok(Values::Shorts(val))
//             }
//             DataType::Long => {
//                 let val: Vec<u32> = Decoded::decode(reader, endian, entry)?;
//                 Ok(Values::Longs(val))
//             }
//             x => Err(DecodeError::from(DecodingError::InvalidDataType(x))),
//         }
//     }
// }

impl Codable for Values {
    type Element = Values;
}

impl Decodable for Values {
    fn decode(val: Values) -> Result<Self, DecodingError> {
        Ok(val)
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

impl Codable for PhotometricInterpretation {
    type Element = u16;
}

impl Decodable for PhotometricInterpretation {
    fn decode(val: u16) -> Result<Self, DecodingError> {
        match val {
            0 => Ok(PhotometricInterpretation::WhiteIsZero),
            1 => Ok(PhotometricInterpretation::BlackIsZero),
            2 => Ok(PhotometricInterpretation::RGB),
            3 => Ok(PhotometricInterpretation::Palette),
            4 => Ok(PhotometricInterpretation::TransparencyMask),
            5 => Ok(PhotometricInterpretation::CMYK),
            6 => Ok(PhotometricInterpretation::YCbCr),
            7 => Ok(PhotometricInterpretation::CIELab),
            n => Err(DecodingError::UnsupportedValue(vec![n])),
        }
    }
}

// impl Decoded for PhotometricInterpretation {
//     fn decode<'a, 'b, 'c, R>(reader: &'a mut R, endian: &'b Endian, entry: &'c Entry) -> Result<Self, DecodeError> 
//         where
//             R: io::Read + io::Seek,
//             'a: 'b,
//             'a: 'c
//     {
//         valid_count!(entry, 1..2, std::any::type_name::<Self>())?;

//         match entry.ty() {
//             DataType::Short => {
//                 let val = entry.field().read_u16(endian)?;
//                 match val {
//                     0 => Ok(PhotometricInterpretation::WhiteIsZero),
//                     1 => Ok(PhotometricInterpretation::BlackIsZero),
//                     2 => Ok(PhotometricInterpretation::RGB),
//                     3 => Ok(PhotometricInterpretation::Palette),
//                     4 => Ok(PhotometricInterpretation::TransparencyMask),
//                     5 => Ok(PhotometricInterpretation::CMYK),
//                     6 => Ok(PhotometricInterpretation::YCbCr),
//                     7 => Ok(PhotometricInterpretation::CIELab),
//                     n => Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n]))),
//                 }
//             }
//             x => Err(DecodeError::from(DecodingError::InvalidDataType(x))),
//         }
//     }
// }

/// Compression scheme used on the image data.
///
/// IFD constructs this with `tag::Compression`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Compression {
    /// LZW compression
    LZW,
}

impl Codable for Option<Compression> {
    type Element = u16;
}

impl Decodable for Option<Compression> {
    fn decode(val: u16) -> Result<Self, DecodingError> {
        match val {
            1 => Ok(None),
            5 => Ok(Some(Compression::LZW)),
            n => Err(DecodingError::UnsupportedValue(vec![n])),
        }
    }
}

// impl Decoded for Option<Compression> {
//     fn decode<'a, 'b, 'c, R>(reader: &'a mut R, endian: &'b Endian, entry: &'c Entry) -> Result<Self, DecodeError> 
//         where
//             R: io::Read + io::Seek,
//             'a: 'b,
//             'a: 'c
//     {
//         valid_count!(entry, 1..2, std::any::type_name::<Self>())?;
//         let val = entry.field().read_u16(endian)?;
//         match val {
//             1 => Ok(None),
//             5 => Ok(Some(Compression::LZW)),
//             n => Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n]))),
//         }
//     }
// }

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

impl<T: Tone> Codable for BitsPerSample<T> {
    type Element = Vec<u16>;
}

impl Decodable for BitsPerSample<DynamicTone> {
    fn decode(val: Vec<u16>) -> Result<Self, DecodingError> {
        match val.len() {
            1 => {
                let val1 = val[0];
                let tone = match val1 {
                    n if n == 8 || n == 16 => DynamicTone::new(n as usize),
                    n => return Err(DecodingError::UnsupportedValue(vec![n]))
                };
    
                Ok(BitsPerSample::C1(tone))
            }
            3 => {
                let val1 = val[0];
                let val2 = val[1];
                let val3 = val[2];

                if val1 != val2 || val1 != val3 {
                    return Err(DecodingError::UnsupportedValue(vec![val1, val2, val3]))
                }

                let tone = match val1 {
                    n if n == 8 || n == 16 => DynamicTone::new(n as usize),
                    n => return Err(DecodingError::UnsupportedValue(vec![n]))
                };

                Ok(BitsPerSample::C3(tone))
            }
            4 => {
                let val1 = val[0];
                let val2 = val[1];
                let val3 = val[2];
                let val4 = val[3];

                if val1 != val2 || val1 != val3 || val1 != val4 {
                    return Err(DecodingError::UnsupportedValue(vec![val1, val2, val3, val4]))
                }

                let tone = match val1 {
                    n if n == 8 || n == 16 => DynamicTone::new(n as usize),
                    n => return Err(DecodingError::UnsupportedValue(vec![n]))
                };

                Ok(BitsPerSample::C4(tone))
            }
            n => unreachable!() // `Possible` limits the number to 1, 3, 4.
        }
    }
}

// impl Decoded for BitsPerSample<DynamicTone> {
//     fn decode<'a, 'b, 'c, R>(reader: &'a mut R, endian: &'b Endian, entry: &'c Entry) -> Result<Self, DecodeError> 
//         where
//             R: io::Read + io::Seek,
//             'a: 'b,
//             'a: 'c
//     {
//         valid_count!(entry, vec![1, 3, 4], std::any::type_name::<Self>())?;

//         if field_is_data_pointer!(reader, endian, entry) {
//             // count = 3 or 4
//             match entry.count() {
//                 3 => {
//                     let val1 = reader.read_u16(&endian)?;
//                     let val2 = reader.read_u16(&endian)?;
//                     let val3 = reader.read_u16(&endian)?;

//                     if val1 != val2 || val1 != val3 {
//                         return Err(DecodeError::from(DecodingError::UnsupportedValue(vec![val1, val2, val3])))
//                     }

//                     let tone = match val1 {
//                         n if n == 8 || n == 16 => DynamicTone::new(n as usize),
//                         n => return Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n])))
//                     };

//                     Ok(BitsPerSample::C3(tone))
//                 }
//                 4 => {
//                     let val1 = reader.read_u16(&endian)?;
//                     let val2 = reader.read_u16(&endian)?;
//                     let val3 = reader.read_u16(&endian)?;
//                     let val4 = reader.read_u16(&endian)?;

//                     if val1 != val2 || val1 != val3 || val1 != val4 {
//                         return Err(DecodeError::from(DecodingError::UnsupportedValue(vec![val1, val2, val3, val4])))
//                     }

//                     let tone = match val1 {
//                         n if n == 8 || n == 16 => DynamicTone::new(n as usize),
//                         n => return Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n])))
//                     };

//                     Ok(BitsPerSample::C4(tone))
//                 }
//                 n => {
//                     let type_name = std::any::type_name::<Self>();
//                     let err = DecodingError::InvalidCount(vec![(n, type_name)]);
                    
//                     Err(DecodeError::from(err))
//                 }
//             }
//         } else {
//             // count = 1
//             let val1 = entry.field().read_u16(&endian)?;

//             let tone = match val1 {
//                 n if n == 8 || n == 16 => DynamicTone::new(n as usize),
//                 n => return Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n])))
//             };

//             Ok(BitsPerSample::C1(tone))
//         }
//     }
// }

/// Difference from the previous pixel
///
/// IFD constructs this with `tag::Predictor`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Predictor {
    None,
    Horizontal,
}

impl Codable for Predictor {
    type Element = u16;
}

impl Decodable for Predictor {
    fn decode(val: u16) -> Result<Self, DecodingError> {
        match val {
            1 => Ok(Predictor::None),
            2 => Ok(Predictor::Horizontal),
            n => Err(DecodingError::UnsupportedValue(vec![n])),
        }
    }
}

// impl Decoded for Predictor {
//     fn decode<'a, 'b, 'c, R>(reader: &'a mut R, endian: &'b Endian, entry: &'c Entry) -> Result<Self, DecodeError> 
//         where
//             R: io::Read + io::Seek,
//             'a: 'b,
//             'a: 'c
//     {
//         valid_count!(entry, 1..2, std::any::type_name::<Self>())?;
//         let val = entry.field().read_u16(endian)?;
//         match val {
//             1 => Ok(Predictor::None),
//             2 => Ok(Predictor::Horizontal),
//             n => Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n]))),
//         }
//     }
// }