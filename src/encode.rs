
use std::io::Write;
use crate::ifd::Entry;
use crate::error::{EncodeResult, EncodeError, EncodeErrorKind};
use crate::byte::Endian;

pub trait EncodeTo: Sized {
    fn encode<W: Write>(encoder: &mut Encoder<W>) -> EncodeResult<()>;
}

#[derive(Debug)]
pub struct Encoder<W> {
    writer: W,
    endian: Endian,
}

impl<W> Encoder<W> {
    pub fn endian(&self) -> Endian {
        self.endian
    }
}

impl<W> Encoder<W> where W: Write {
    /// 
    /// 
    /// write from A to B while deciding the position correctly
    pub fn write_up(&mut self) -> EncodeResult<()> {

        unimplemented!()
    }
}