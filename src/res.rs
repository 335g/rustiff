
use std::marker::PhantomData;
use crate::error::DecodeError;

pub struct Result<V, T = (), DecodeError> {
    res: Result<V, DecodeError>,
    ghost: PhantomData<fn() -> T>
}

impl<V, T> Result<V, T, DecodeError> {
    pub(crate) fn success(val: V) -> Self {
        Self {
            res: Ok(val),
            ghost: PhantomData
        }
    }

    pub(crate) fn failure(err: DecodeError) -> Self {
        Self {
            res: Err(err),
            ghost: PhantomData
        }
    }
}