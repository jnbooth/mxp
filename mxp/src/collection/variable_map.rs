use std::collections::HashMap;
use std::iter::FusedIterator;
use std::ops::{Deref, DerefMut};
use std::slice;

use crate::keyword::EntityKeyword;
use std::collections::hash_map::Entry;

use super::published_entities::{PublishedEntities, PublishedEntity};
use enumeration::EnumSet;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VariableMap {
    inner: HashMap<String, String>,
    published: PublishedEntities,
}

impl Deref for VariableMap {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for VariableMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct EntityEntry<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
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

    pub fn set<'a>(
        &'a mut self,
        key: &'a str,
        value: &str,
        desc: Option<String>,
        keywords: EnumSet<EntityKeyword>,
    ) -> Option<EntityEntry<'a>> {
        if keywords.contains(EntityKeyword::Delete) {
            return self.remove(key).map(|_| EntityEntry {
                name: key,
                value: None,
            });
        }
        if keywords.contains(EntityKeyword::Private) {
            self.published.remove(key);
        } else if keywords.contains(EntityKeyword::Publish) {
            self.published.insert(key.to_owned(), desc);
        }
        let entity = match self.inner.entry(key.to_owned()) {
            Entry::Vacant(_) if keywords.contains(EntityKeyword::Remove) => return None,
            Entry::Vacant(entry) => entry.insert(value.to_owned()),
            Entry::Occupied(entry) if keywords.contains(EntityKeyword::Remove) => {
                if entry.get() == value {
                    entry.remove();
                    return Some(EntityEntry {
                        name: key,
                        value: None,
                    });
                }
                let entity = entry.into_mut();
                *entity = entity
                    .split('|')
                    .filter(|item| *item != value)
                    .collect::<Vec<_>>()
                    .join("|");
                entity
            }
            Entry::Occupied(entry) if keywords.contains(EntityKeyword::Add) => {
                let entity = entry.into_mut();
                entity.reserve(value.len() + 1);
                entity.push('|');
                entity.push_str(value);
                entity
            }
            Entry::Occupied(entry) => {
                let entity = entry.into_mut();
                if entity == value {
                    return None;
                }
                *entity = value.to_owned();
                entity
            }
        };
        Some(EntityEntry {
            name: key,
            value: Some(entity),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityInfo<'a> {
    name: &'a str,
    desc: &'a str,
    value: &'a str,
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct PublishedIter<'a> {
    inner: slice::Iter<'a, PublishedEntity>,
    map: &'a HashMap<String, String>,
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
