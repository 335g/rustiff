
use std::ops::RangeBounds;

use crate::error::DecodingError;
use crate::element::Element;

pub trait Decoded: Sized {
    type Element: Element;
    type Poss: Possible;

    const POSSIBLE_COUNT: Self::Poss;

    fn decoded(elements: Vec<Self::Element>) -> Result<Self, DecodingError>;
}

pub trait Possible {
    fn contains_item(&self, item: &usize) -> bool;
}

impl Possible for usize {
    fn contains_item(&self, item: &usize) -> bool {
        *self == *item
    }
}

impl Possible for std::ops::Range<usize> {
    fn contains_item(&self, item: &usize) -> bool {
        self.contains(item)
    }
}

impl Possible for std::ops::RangeFrom<usize> {
    fn contains_item(&self, item: &usize) -> bool {
        self.contains(item)
    }
}

impl Possible for std::ops::RangeTo<usize> {
    fn contains_item(&self, item: &usize) -> bool {
        self.contains(item)
    }
}

impl Possible for std::ops::RangeFull {
    fn contains_item(&self, item: &usize) -> bool {
        self.contains(item)
    }
}

impl Possible for std::ops::RangeToInclusive<usize> {
    fn contains_item(&self, item: &usize) -> bool {
        self.contains(item)
    }
}

impl Possible for std::ops::RangeInclusive<usize> {
    fn contains_item(&self, item: &usize) -> bool {
        self.contains(item)
    }
}

impl <const N: usize> Possible for [usize; N] {
    fn contains_item(&self, item: &usize) -> bool {
        for i in self {
            if i == item {
                return true
            }
        }

        false
    }
}

