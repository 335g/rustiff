
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::HashMap;

use tag::TagKind;

#[derive(Debug, Clone, Copy)]
pub enum DataType {
    Byte,
    Short,
    Long,
    Rational,
    Unknown(u16),
}

impl DataType {
    pub fn from_u16(n: u16) -> DataType {
        match n {
            1 => DataType::Byte,
            3 => DataType::Short,
            4 => DataType::Long,
            5 => DataType::Rational,
            n => DataType::Unknown(n),
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct IFD {
    from: u32,
    map: HashMap<TagKind, Entry>,
}

impl IFD {
    pub fn new(from: u32) -> IFD {
        IFD {
            from: from,
            map: HashMap::new(),
        }
    }

    pub fn from(&self) -> u32 {
        self.from
    }

    pub fn insert(&mut self, k: TagKind, v: Entry) -> Option<Entry> {
        self.map.insert(k, v)
    }
    
    #[inline]
    pub fn get(&self, k: &TagKind) -> Option<&Entry> {
        self.map.get(k)
    }
}

