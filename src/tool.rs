
#![allow(missing_docs)]

pub(crate) trait HasValue {
    type Value;
}

pub(crate) struct Empty;
pub(crate) struct Filled<T>(pub T);

impl HasValue for Empty {
    type Value = ();
}

impl<T> HasValue for Filled<T> {
    type Value = T;
}
