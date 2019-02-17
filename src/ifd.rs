
use std::collections::HashMap;
use std::fmt::{
    self,
    Display,
};
use tag::{
    TagType,
    AnyTag,
    ImpossibleTag,
};
use error::DecodeError;

/// DataType in IFD
#[derive(Debug, Clone, Copy)]
pub enum DataType {
    Byte,
    Short,
    Long,
    Rational,
    Unknown(u16),
}

impl From<u16> for DataType {
    fn from(n: u16) -> DataType {
        match n {
            1 => DataType::Byte,
            3 => DataType::Short,
            4 => DataType::Long,
            5 => DataType::Rational,
            n => DataType::Unknown(n),
        }
    }
}

/// IFD entry
#[derive(Debug, Clone, Fail)]
pub struct Entry {
    datatype: DataType,
    count: u32,
    offset: [u8; 4],
}

impl Entry {
    pub fn new(datatype: DataType, count: u32, offset: [u8; 4]) -> Entry {
        Entry {
            datatype: datatype,
            count: count,
            offset: offset,
        }
    }

    pub fn datatype(&self) -> DataType {
        self.datatype
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn offset(&self) -> &[u8] {
        &self.offset
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Entry(datatype: {:?}, count: {}, offset: {:?}", self.datatype, self.count, self.offset)
    }
}

/// IFD (Image File Directory)
#[derive(Debug, Clone)]
pub struct IFD(HashMap<AnyTag, Entry>);

impl IFD {
    #[allow(missing_docs)]
    #[inline]
    pub fn new() -> IFD {
        IFD(HashMap::new())
    }
    
    // TODO: replace EncodeError
    /// Insert an `ifd::Entry` into the IFD.
    /// 
    /// Return value behavior confirms to `collections::HashMap`.
    /// If the map did not have the tag present, `Option<Entry>::None` returned.
    /// If the map did have this tag present, the `Entry` is updated, and the old `Entry` is returned.
    #[inline]
    pub fn insert<T: TagType>(&mut self, tag: T, entry: Entry) -> Result<Option<Entry>, ImpossibleTag<T>> {
        let anytag = AnyTag::try_from(tag)?;
        
        Ok(self.insert_anytag(anytag, entry))
    }
    
    #[allow(missing_docs)]
    #[inline]
    pub(crate) fn insert_anytag(&mut self, tag: AnyTag, entry: Entry) -> Option<Entry> {
        self.0.insert(tag, entry)
    }
    
    /// Returns the reference to the `ifd::Entry` to the tag.
    #[inline]
    pub fn get<T: TagType>(&self, tag: T) -> Result<Option<&Entry>, DecodeError> {
        let anytag = AnyTag::try_from(tag)?;

        Ok(self.get_anytag(anytag))
    }
    
    #[allow(missing_docs)]
    #[inline]
    pub(crate) fn get_anytag(&self, k: AnyTag) -> Option<&Entry> {
        self.0.get(&k)
    }
}

