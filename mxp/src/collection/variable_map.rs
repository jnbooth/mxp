use std::iter::FusedIterator;
use std::ops::{Deref, DerefMut};
use std::slice;

use crate::keyword::EntityKeyword;

use super::published_entities::{PublishedEntities, PublishedEntity};
use casefold::ascii::CaseFoldMap;
use enumeration::EnumSet;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VariableMap {
    inner: CaseFoldMap<String, String>,
    published: PublishedEntities,
}

impl Deref for VariableMap {
    type Target = CaseFoldMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for VariableMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl VariableMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.published.clear();
    }

    pub fn published(&self) -> PublishedIter {
        PublishedIter {
            inner: self.published.iter(),
            map: &self.inner,
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.published.remove(key);
        self.inner.remove(key)
    }

    pub fn set(
        &mut self,
        key: &str,
        value: &str,
        desc: Option<String>,
        keywords: EnumSet<EntityKeyword>,
    ) -> bool {
        if keywords.contains(EntityKeyword::Delete) {
            self.inner.remove(key);
            self.published.remove(key);
            return false;
        }
        if keywords.contains(EntityKeyword::Private) {
            self.published.remove(key);
        } else if keywords.contains(EntityKeyword::Publish) {
            self.published.insert(key.to_owned(), desc)
        }
        if keywords.contains(EntityKeyword::Remove) {
            return self.remove_list_item(key, value);
        }
        if keywords.contains(EntityKeyword::Add) {
            self.add_list_item(key, value);
            return true;
        }
        self.inner.insert(key.to_owned(), value.to_owned());
        true
    }

    pub fn add_list_item(&mut self, key: &str, value: &str) {
        let entity = match self.inner.get_mut(key) {
            Some(entity) => entity,
            None => {
                self.inner.insert(key.to_owned(), value.to_owned());
                return;
            }
        };
        entity.reserve(value.len() + 1);
        entity.push('|');
        entity.push_str(value);
    }

    pub fn remove_list_item(&mut self, key: &str, value: &str) -> bool {
        let entity = match self.inner.get_mut(key) {
            Some(entity) => entity,
            None => return false,
        };
        *entity = entity
            .split('|')
            .filter(|item| *item != value)
            .collect::<Vec<_>>()
            .join("|");
        true
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityInfo<'a> {
    name: &'a str,
    desc: &'a str,
    value: &'a str,
}

pub struct PublishedIter<'a> {
    inner: slice::Iter<'a, PublishedEntity>,
    map: &'a CaseFoldMap<String, String>,
}

impl<'a> PublishedIter<'a> {
    fn lookup(&self, entity: &'a PublishedEntity) -> EntityInfo<'a> {
        EntityInfo {
            name: &entity.name,
            desc: &entity.desc,
            value: match self.map.get(&entity.name) {
                Some(entity) => entity.as_str(),
                None => "",
            },
        }
    }
}

impl<'a> Iterator for PublishedIter<'a> {
    type Item = EntityInfo<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|entity| self.lookup(entity))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|entity| self.lookup(entity))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n).map(|entity| self.lookup(entity))
    }
}

impl<'a> FusedIterator for PublishedIter<'a> {}

impl<'a> ExactSizeIterator for PublishedIter<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> DoubleEndedIterator for PublishedIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|entity| self.lookup(entity))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth_back(n).map(|entity| self.lookup(entity))
    }
}
