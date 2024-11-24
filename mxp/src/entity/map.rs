use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use super::global::{CHARS, GLOBAL_ENTITIES, MIN_CHAR};
use crate::keyword::EntityKeyword;
use crate::parser::{Error, ErrorKind};
use std::collections::hash_map::Entry;

use super::iter::PublishedIter;

use super::entity::Entity;
use enumeration::EnumSet;

pub struct EntityEntry<'a> {
    pub name: &'a str,
    pub value: Option<&'a Entity>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityMap {
    inner: HashMap<String, Entity>,
    globals: HashSet<String>,
}

impl Deref for EntityMap {
    type Target = HashMap<String, Entity>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for EntityMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl EntityMap {
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
                    entity.value.clear();
                    entity.value.push_str(value);
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

    pub fn decode_entity(&self, key: &str) -> crate::Result<Option<&str>> {
        let Some(code) = key.strip_prefix('#') else {
            return Ok(self.inner.get(key).map(|entity| entity.value.as_ref()));
        };
        let id = match code.strip_prefix('x') {
            Some(hex) => usize::from_str_radix(hex, 16),
            None => code.parse::<usize>(),
        }
        .map_err(|_| Error::new(key, ErrorKind::InvalidEntityNumber))?;
        match id.checked_sub(MIN_CHAR).and_then(|id| CHARS.get(id..=id)) {
            None => Err(Error::new(key, ErrorKind::DisallowedEntityNumber)),
            some => Ok(some),
        }
    }
}
