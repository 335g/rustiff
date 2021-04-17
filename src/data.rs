use std::convert::TryFrom;

use crate::error::{DecodeError, DecodeResult, DecodingError};

#[derive(Debug)]
pub enum Data {
    U8(Vec<u8>),
    U16(Vec<u16>),
}

/// Data type in IFD
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

    fn try_from(n: u16) -> DecodeResult<Self> {
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
            n => return Err(DecodeError::from(DecodingError::UnsupportedValue(vec![n]))),
        };

        Ok(ty)
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    ty: DataType,
    count: usize,
    field: [u8; 4],
}

impl Entry {
    pub(crate) fn new(ty: DataType, count: usize, field: [u8; 4]) -> Entry {
        Entry { ty, count, field }
    }

    pub fn ty(&self) -> DataType {
        self.ty
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn field(&self) -> &[u8] {
        &self.field
    }

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
