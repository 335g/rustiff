use std::convert::TryFrom;
use crate::error::DecodingError;

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
    type Error = DecodingError;

    fn try_from(n: u16) -> Result<Self, DecodingError> {
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
            n => return Err(DecodingError::UnsupportedValueForDataType(n)),
        };

        Ok(ty)
    }
}