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
                description: description.unwrap_or_default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_value<'a>(map: &'a EntityMap, key: &str) -> Option<&'a str> {
        let entity = map.get(key)?;
        Some(entity.value.as_str())
    }

    #[test]
    fn hash_entities_are_global() {
        assert!(EntityMap::new().is_global("#e"));
    }

    #[test]
    fn globals_are_not_added_by_default() {
        assert!(!EntityMap::new().is_global("lt"));
    }

    #[test]
    fn globals_are_added_by_method() {
        let mut map = EntityMap::new();
        map.add_globals();
        assert!(map.is_global("lt"));
    }

    #[test]
    fn set_new() {
        let mut map = EntityMap::new();
        map.set("key", "value", None, EnumSet::new());
        assert_eq!(get_value(&map, "key"), Some("value"));
    }

    #[test]
    fn set_delete() {
        let mut map = EntityMap::new();
        map.set("key", "value", None, EnumSet::new());
        map.set("key", "", None, enums![EntityKeyword::Delete]);
        assert_eq!(get_value(&map, "key"), None);
    }

    #[test]
    fn set_replace() {
        let mut map = EntityMap::new();
        map.set("key", "value", Some("desc1".to_owned()), EnumSet::new());
        map.set(
            "key",
            "",
            Some("desc2".to_owned()),
            enums![EntityKeyword::Publish],
        );
        assert_eq!(
            map.get("key"),
            Some(&Entity {
                value: String::new(),
                published: true,
                description: "desc2".to_owned()
            })
        );
    }

    #[test]
    fn set_add_and_remove() {
        let mut map = EntityMap::new();
        map.set("key", "value1", None, enums![EntityKeyword::Add]);
        map.set("key", "value2", None, enums![EntityKeyword::Add]);
        map.set("key", "value3", None, enums![EntityKeyword::Add]);
        map.set("key", "value2", None, enums![EntityKeyword::Remove]);
        map.set("key", "x", None, enums![EntityKeyword::Remove]);
        assert_eq!(get_value(&map, "key"), Some("value1|value3"));
    }

    #[test]
    fn decode_entity_matched() {
        let mut map = EntityMap::new();
        map.set("key1", "value1", None, EnumSet::new());
        map.set("key2", "value2", None, EnumSet::new());
        assert_eq!(map.decode_entity("key1"), Ok(Some("value1")));
    }

    #[test]
    fn decode_entity_unmatched() {
        let mut map = EntityMap::new();
        map.set("key2", "value2", None, EnumSet::new());
        assert_eq!(map.decode_entity("key1"), Ok(None));
    }

    #[test]
    fn decode_decimal() {
        assert_eq!(EntityMap::new().decode_entity("#32"), Ok(Some("\x20")));
    }

    #[test]
    fn decode_hex() {
        assert_eq!(EntityMap::new().decode_entity("#x7F"), Ok(Some("\x7f")));
    }

    #[test]
    fn decode_invalid_number() {
        assert_eq!(
            EntityMap::new().decode_entity("#x7z"),
            Err(Error::new("#x7z", ErrorKind::InvalidEntityNumber))
        );
    }

    #[test]
    fn decode_below_range() {
        assert_eq!(
            EntityMap::new().decode_entity("#10"),
            Err(Error::new("#10", ErrorKind::DisallowedEntityNumber))
        );
    }

    #[test]
    fn decode_above_range() {
        assert_eq!(
            EntityMap::new().decode_entity("#x90"),
            Err(Error::new("#x90", ErrorKind::DisallowedEntityNumber))
        );
    }
}
