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
