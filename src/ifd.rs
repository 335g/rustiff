use crate::tag::{AnyTag, Tag};
use crate::{data::Entry, error::TagError};

use std::collections::{self, HashMap};

/// IFD (Image File Directory)
#[derive(Debug, Clone)]
pub struct ImageFileDirectory(HashMap<AnyTag, Entry>);

impl ImageFileDirectory {
    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub(crate) fn new() -> Self {
        ImageFileDirectory(HashMap::new())
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub(crate) fn insert<T: Tag>(&mut self, entry: Entry) -> Result<Option<Entry>, TagError<T>> {
        let anytag = AnyTag::try_from::<T>()?;
        let res = self.insert_tag(anytag, entry);

        Ok(res)
    }

    #[inline]
    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub(crate) fn insert_tag(&mut self, tag: AnyTag, entry: Entry) -> Option<Entry> {
        self.0.insert(tag, entry)
    }

    #[inline]
    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn get_tag(&self, tag: &AnyTag) -> Option<&Entry> {
        self.0.get(tag)
    }

    #[allow(dead_code)]
    #[allow(missing_docs)]
    pub fn keys(&self) -> Keys<'_> {
        Keys {
            inner: self.0.keys(),
        }
    }
}

pub struct Keys<'a> {
    inner: collections::hash_map::Keys<'a, AnyTag, Entry>,
}

impl<'a> Iterator for Keys<'a> {
    type Item = &'a AnyTag;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
