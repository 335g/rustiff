use crate::error::{DecodeError, DecodeErrorKind};
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Byte,
    Ascii,
    Short,
    Long,
    Rational,
    SByte,
    Undefined,
    SShort,
    SLong,
    SRational,
    Float,
    Double,
}

impl TryFrom<u16> for DataType {
    type Error = DecodeError;

    fn try_from(n: u16) -> Result<Self, DecodeError> {
        use DataType::*;

        let ty = match n {
            1 => Byte,
            2 => Ascii,
            3 => Short,
            4 => Long,
            5 => Rational,
            6 => SByte,
            7 => Undefined,
            8 => SShort,
            9 => SLong,
            10 => SRational,
            11 => Float,
            12 => Double,
            n => return Err(DecodeError::new(DecodeErrorKind::UnsupportedDataType(n))),
        };

        Ok(ty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rational<T> {
    pub numerator: T,
    pub denominator: T,
}

impl<T> Rational<T> {
    pub fn new(numerator: T, denominator: T) -> Self {
        Self { numerator, denominator }
    }
}

#[derive(Debug)]
pub enum AnyElements {
    Byte(Vec<u8>),
    Ascii(Vec<char>),
    Short(Vec<u16>),
    Long(Vec<u32>),
    Rational(Vec<Rational<u32>>),
    SByte(Vec<i8>),
    Undefined(Vec<u8>),
    SShort(Vec<i16>),
    SLong(Vec<i32>),
    SRational(Vec<Rational<i32>>),
    Float(Vec<f32>),
    Double(Vec<f64>),
}

#[derive(Debug, Clone)]
pub struct Entry {
    ty: DataType,
    count: usize,
    field: [u8; 4],
}

impl Entry {
    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub(crate) fn new(ty: DataType, count: usize, field: [u8; 4]) -> Entry {
        Entry { ty, count, field }
    }

    #[inline]
    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn ty(&self) -> DataType {
        self.ty
    }

    #[inline]
    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub const fn count(&self) -> usize {
        self.count
    }

    #[inline]
    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn field(&self) -> &[u8] {
        &self.field
    }

    #[inline]
    #[allow(missing_docs)]
    pub fn field_mut(&mut self) -> &mut [u8] {
        &mut self.field
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn overflow(&self) -> bool {
        use DataType::*;

        match self.ty {
            Byte | Ascii | SByte | Undefined => self.count > 4,
            Short | SShort => self.count > 2,
            Long | SLong | Float => self.count > 1,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub enum ImageData {
    U8(Vec<u8>),
    U16(Vec<u16>),
}
