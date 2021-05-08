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
