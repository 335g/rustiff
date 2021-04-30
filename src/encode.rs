
use crate::decode::Decoded;

pub trait Encoded<T: Decoded>: Sized {
    fn encoded(val: T) -> Self;
}
