use std::collections::{hash_map, HashMap, HashSet};
use std::iter::FusedIterator;
use std::ops::{Deref, DerefMut};

use super::global_entities::GLOBAL_ENTITIES;
use crate::keyword::EntityKeyword;
use std::collections::hash_map::Entry;

use enumeration::EnumSet;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    pub value: String,
    pub published: bool,
    pub description: String,
}

impl Entity {
    pub const fn new(value: String) -> Self {
        Self {
            value,
            published: false,
            description: String::new(),
        }
    }

    pub fn apply_keywords(&mut self, keywords: EnumSet<EntityKeyword>) {
        if keywords.contains(EntityKeyword::Private) {
            self.published = false;
        } else if keywords.contains(EntityKeyword::Publish) {
            self.published = true;
        }
    }

    pub fn add(&mut self, value: &str) {
        self.value.reserve(value.len() + 1);
        self.value.push('|');
        self.value.push_str(value);
    }

    pub fn remove(&mut self, value: &str) {
        self.value = self
            .value
            .split('|')
            .filter(|item| *item != value)
            .collect::<Vec<_>>()
            .join("|");
    }
}

pub struct EntityEntry<'a> {
    pub name: &'a str,
    pub value: Option<&'a Entity>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VariableMap {
    inner: HashMap<String, Entity>,
    globals: HashSet<String>,
}

impl Deref for VariableMap {
    type Target = HashMap<String, Entity>;

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
        self.globals.clear();
    }

    pub fn remove(&mut self, key: &str) -> Option<Entity> {
        self.inner.remove(key)
    }

    pub fn published(&self) -> PublishedIter {
        PublishedIter {
            inner: self.inner.iter(),
        }
    }

    pub fn is_global(&self, key: &str) -> bool {
        key.starts_with('#') || self.globals.contains(key)
    }

    pub fn add_globals(&mut self) {
        if !self.globals.is_empty() {
            return;
        }
        for (key, val) in GLOBAL_ENTITIES {
            self.inner.insert(
                (*key).to_owned(),
                Entity {
                    value: (*val).to_owned(),
                    published: false,
                    description: String::new(),
                },
            );
            self.globals.insert((*key).to_owned());
        }
    }

    pub fn set<'a>(
        &'a mut self,
        key: &'a str,
        value: &str,
        description: Option<String>,
        keywords: EnumSet<EntityKeyword>,
    ) -> Option<EntityEntry<'a>> {
        if keywords.contains(EntityKeyword::Delete) {
            return self.remove(key).map(|_| EntityEntry {
                name: key,
                value: None,
            });
        }
        let entity = match self.inner.entry(key.to_owned()) {
            Entry::Vacant(_) if keywords.contains(EntityKeyword::Remove) => return None,
            Entry::Vacant(entry) => entry.insert(Entity {
                value: value.to_owned(),
                published: keywords.contains(EntityKeyword::Publish),
                description: String::new(),
            }),
            Entry::Occupied(entry) if keywords.contains(EntityKeyword::Remove) => {
                if entry.get().value == value {
                    entry.remove();
                    return Some(EntityEntry {
                        name: key,
                        value: None,
                    });
                }
                let entity = entry.into_mut();
                entity.remove(value);
                entity.apply_keywords(keywords);
                entity
            }
            Entry::Occupied(entry) if keywords.contains(EntityKeyword::Add) => {
                let entity = entry.into_mut();
                entity.add(value);
                entity.apply_keywords(keywords);
                entity
            }
            Entry::Occupied(entry) => {
                let entity = entry.into_mut();
                let description_unchanged = match description {
                    Some(description) if entity.description != description => {
                        entity.description = description;
                        false
                    }
                    _ => true,
                };
                if description_unchanged && entity.value == value {
                    let old_published = entity.published;
                    entity.apply_keywords(keywords);
                    if entity.published == old_published {
                        return None;
                    }
                } else {
                    entity.value = value.to_owned();
                    entity.apply_keywords(keywords);
                }
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
    description: &'a str,
    value: &'a str,
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct PublishedIter<'a> {
    inner: hash_map::Iter<'a, String, Entity>,
}

impl<'a> Iterator for PublishedIter<'a> {
    type Item = EntityInfo<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for (name, entity) in &mut self.inner {
            if entity.published {
                return Some(EntityInfo {
                    name,
                    description: entity.description.as_str(),
                    value: entity.value.as_str(),
                });
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.inner.len()))
    }

    fn count(self) -> usize {
        self.inner.filter(|item| item.1.published).count()
    }
}

impl<'a> FusedIterator for PublishedIter<'a> {}
