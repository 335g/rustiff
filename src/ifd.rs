use crate::error::{DecodeError, DecodeResult, DecodeValueErrorDetail};
use crate::tag::{AnyTag, Tag};
use std::collections::HashMap;
use std::convert::From;
use std::convert::TryFrom;

/// Data type in IFD
#[derive(Debug, Clone, Copy)]
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
                return Err(DecodeError::from(DecodeValueErrorDetail::InvalidValue(
                    vec![n as u32],
                )))
            }
        };

        Ok(ty)
    }
}

impl DataType {
    fn overflow(&self, count: u32) -> bool {
        use DataType::*;

        match self {
            Byte | Ascii | SByte | Undefined => count > 4,
            Short | SShort => count > 2,
            Long | SLong | Float => count > 1,
            _ => true,
        }
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
        self.ty.overflow(self.count)
    }
}

/// IFD (Image File Directory)
#[derive(Debug, Clone)]
pub struct IFD(HashMap<AnyTag, Entry>);

impl IFD {
    #[allow(missing_docs)]
    pub(crate) fn new() -> IFD {
        IFD(HashMap::new())
    }

    /// Insert an `ifd::Entry` into the IFD
    ///
    /// Return value behavior confirms to `collections::HashMap`.
    /// If the map did not have the tag present, `Option<Entry>::None` returned.
    /// If the map did have this tag present, the `Entry` is updated, and the old `Entry` is returned.
    pub fn insert<T: Tag>(&mut self, entry: Entry) -> DecodeResult<Option<Entry>> {
        let anytag = AnyTag::try_from::<T>()?;
        let res = self.insert_tag(anytag, entry);

        Ok(res)
    }

    #[allow(missing_docs)]
    #[inline]
    pub(crate) fn insert_tag(&mut self, tag: AnyTag, entry: Entry) -> Option<Entry> {
        self.0.insert(tag, entry)
    }

    /// Returns the reference to the `ifd::Entry` to the tag.
    #[inline]
    pub fn get<T: Tag>(&self) -> DecodeResult<Option<&Entry>> {
        let anytag = AnyTag::try_from::<T>()?;
        let res = self.get_tag(anytag);

        Ok(res)
    }

    #[allow(missing_docs)]
    #[inline]
    pub(crate) fn get_tag(&self, tag: AnyTag) -> Option<&Entry> {
        self.0.get(&tag)
    }
}
