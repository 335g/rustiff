use crate::{decode::Decoded, element::Endian};

pub trait Encoded<T: Decoded>: Sized {
    fn encoded(val: T) -> Self;
}

#[derive(Debug)]
pub struct Encoder<W> {
    writer: W,
    endian: Endian,
}
