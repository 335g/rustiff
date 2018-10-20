
#![allow(missing_docs)]

pub trait HasValue {
    type Value;
}

pub struct Empty;
pub struct Filled<T>(pub T);

impl HasValue for Empty {
    type Value = ();
}

impl<T> HasValue for Filled<T> {
    type Value = T;
}
