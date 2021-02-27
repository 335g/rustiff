use crate::data::Entry;
use crate::error::{DecodeError, DecodeResult, DecodeValueError};
use crate::tag::{self, AnyTag, Tag};
use crate::val::Value;
use std::convert::{From, TryFrom};
use std::marker::PhantomData;
use std::{
    collections::HashMap,
    hash,
    iter::FusedIterator,
    ops::{Deref, DerefMut},
};

/// IFD (Image File Directory)
#[derive(Debug, Clone)]
pub struct ImageFileDirectory(HashMap<AnyTag, Entry>);

impl ImageFileDirectory {
    #[allow(missing_docs)]
    pub(crate) fn new() -> Self {
        ImageFileDirectory(HashMap::new())
    }

    /// Insert an `ifd::Entry` into the IFD
    ///
    /// Return value behavior confirms to `collections::HashMap`.
    /// If the map did not have the tag present, `Option<Entry>::None` returned.
    /// If the map did have this tag present, the `Entry` is updated, and the old `Entry` is returned.
    pub(crate) fn insert<T: Tag>(&mut self, entry: Entry) -> DecodeResult<Option<Entry>> {
        let anytag = AnyTag::try_from::<T>()?;
        let res = self.insert_tag(anytag, entry);

        Ok(res)
    }

    #[allow(missing_docs)]
    #[inline]
    pub(crate) fn insert_tag(&mut self, tag: AnyTag, entry: Entry) -> Option<Entry> {
        self.0.insert(tag, entry)
    }

    /// Returns the reference to the `ifd::Entry` to the tag.
    #[inline]
    pub(crate) fn get<T: Tag>(&self) -> DecodeResult<Option<&Entry>> {
        let anytag = AnyTag::try_from::<T>()?;
        let res = self.get_tag(anytag);

        Ok(res)
    }

    #[allow(missing_docs)]
    #[inline]
    pub(crate) fn get_tag(&self, tag: AnyTag) -> Option<&Entry> {
        self.0.get(&tag)
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            base: self.0.iter(),
        }
    }

    pub fn tags(&self) -> Tags<'_> {
        Tags { inner: self.iter() }
    }

    pub fn entries(&self) -> Entries<'_> {
        Entries { inner: self.iter() }
    }
}

pub struct Iter<'a> {
    base: std::collections::hash_map::Iter<'a, AnyTag, Entry>,
}

impl<'a> Deref for Iter<'a> {
    type Target = std::collections::hash_map::Iter<'a, AnyTag, Entry>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a> DerefMut for Iter<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub struct Tags<'a> {
    inner: Iter<'a>,
}

impl<'a> Iterator for Tags<'a> {
    type Item = &'a AnyTag;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> ExactSizeIterator for Tags<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> FusedIterator for Tags<'a> {}

pub struct Entries<'a> {
    inner: Iter<'a>,
}

impl<'a> Iterator for Entries<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> ExactSizeIterator for Entries<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> FusedIterator for Entries<'a> {}
