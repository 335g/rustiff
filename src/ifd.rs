use std::collections::HashMap;
use std::fmt::{
    self,
    Display,
};
use tag::{
    TagType,
    AnyTag,
    TagError,
};

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

#[derive(Debug, Clone)]
pub struct IFD(HashMap<AnyTag, Entry>);

impl IFD {
    #[inline]
    pub fn new() -> IFD {
        IFD(HashMap::new())
    }
    
    #[inline]
    pub fn insert<T: TagType>(&mut self, tag: T, entry: Entry) -> Result<Option<Entry>, TagError<T>> {
        let anytag = AnyTag::try_from(tag)?;
        
        Ok(self.insert_anytag(anytag, entry))
    }
    
    #[inline]
    pub fn insert_anytag(&mut self, tag: AnyTag, entry: Entry) -> Option<Entry> {
        self.0.insert(tag, entry)
    }
    
    #[inline]
    pub fn get<T: TagType>(&self, tag: T) -> Result<Option<&Entry>, TagError<T>> {
        let anytag = AnyTag::try_from(tag)?;

        Ok(self.get_anytag(anytag))
    }

    #[inline]
    pub fn get_anytag(&self, k: AnyTag) -> Option<&Entry> {
        self.0.get(&k)
    }
}

