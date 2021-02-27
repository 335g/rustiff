use std::convert::TryFrom;

use crate::{DecodeError, DecodeResult, DecodeValueError};

pub enum Data {
    Byte(Vec<u8>),
    Short(Vec<u16>),
}

impl Data {
    pub fn byte_with(size: usize) -> Data {
        Data::Byte(vec![0; size])
    }

    pub fn short_with(size: usize) -> Data {
        Data::Short(vec![0; size])
    }
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
            n => {
                return Err(DecodeError::from(DecodeValueError::InvalidValue(vec![
                    n as u32,
                ])))
            }
        };

        Ok(ty)
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    ty: DataType,
    count: u32,
    field: [u8; 4],
}

impl Entry {
    pub(crate) fn new(ty: DataType, count: u32, field: [u8; 4]) -> Entry {
        Entry { ty, count, field }
    }

    pub fn ty(&self) -> DataType {
        self.ty
    }

    pub fn count(&self) -> u32 {
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
