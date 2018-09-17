
use std::collections::HashMap;
use std::fmt::{
    self,
    Display,
};
use tag::{
    TagType,
    AnyTag,
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
pub struct IFD(HashMap<u16, Entry>);

impl IFD {
    pub fn new() -> IFD {
        IFD(HashMap::new())
    }

    pub fn insert<T: TagType>(&mut self, k: T, v: Entry) -> Option<Entry> {
        self.0.insert(k.id(), v)
    }

    pub fn insert_anytag(&mut self, k: AnyTag, v: Entry) -> Option<Entry> {
        self.0.insert(k.id(), v)
    }
    
    #[inline]
    pub fn get<T: TagType>(&self, k: T) -> Option<&Entry> {
        self.0.get(&k.id())
    }
}

