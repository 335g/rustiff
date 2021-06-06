use std::io;

use crate::{decode::Decoded, element::Endian, error::EncodeResult, image::Image};

pub trait Encoded<T: Decoded>: Sized {
    fn encoded(val: T) -> Self;
}

#[derive(Debug)]
pub struct EncoderBuilder {}

#[derive(Debug)]
pub struct Encoder<W> {
    writer: W,
    endian: Endian,
}

impl<W> Encoder<W> {}

impl<W> Encoder<W>
where
    W: io::Write + io::Seek,
{
    pub fn encode_image(&mut self, img: Image) -> EncodeResult<()> {
        todo!()
    }

    pub fn finish(&mut self) -> EncodeResult<()> {
        todo!()
    }
}
