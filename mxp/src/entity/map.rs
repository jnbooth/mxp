use std::collections::HashMap;

use super::global::{CHARS, GLOBAL_ENTITIES, MIN_CHAR};
use crate::keyword::EntityKeyword;
use crate::parser::{Error, ErrorKind};
use std::collections::hash_map::Entry;

use super::iter::PublishedIter;

use super::entity::Entity;
use flagset::FlagSet;

pub struct EntityEntry<'a> {
    pub name: &'a str,
    pub value: Option<&'a Entity>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityMap {
    inner: HashMap<String, Entity>,
    globals: HashMap<&'static str, &'static str>,
}

impl EntityMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_globals() -> Self {
        Self {
            inner: HashMap::new(),
            globals: GLOBAL_ENTITIES.iter().copied().collect(),
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    fn guard_global(&self, key: &str) -> crate::Result<()> {
        if self.is_global(key) {
            return Err(crate::Error::new(key, ErrorKind::CannotRedefineEntity));
        }
        Ok(())
    }

    pub fn remove(&mut self, key: &str) -> crate::Result<Option<Entity>> {
        self.guard_global(key)?;
        Ok(self.inner.remove(key))
    }

    pub fn is_global(&self, key: &str) -> bool {
        key.starts_with('#') || self.globals.contains_key(key)
    }

    pub fn published(&self) -> PublishedIter {
        PublishedIter {
            inner: self.inner.iter(),
        }
    }

    pub fn set<'a, T: Into<FlagSet<EntityKeyword>>>(
        &'a mut self,
        key: &'a str,
        value: &str,
        description: Option<String>,
        keywords: T,
    ) -> crate::Result<Option<EntityEntry<'a>>> {
        // Reduce monomorphization
        fn inner<'a>(
            map: &'a mut EntityMap,
            key: &'a str,
            value: &str,
            description: Option<String>,
            keywords: FlagSet<EntityKeyword>,
        ) -> crate::Result<Option<EntityEntry<'a>>> {
            map.guard_global(key)?;
            if keywords.contains(EntityKeyword::Delete) {
                return Ok(map.inner.remove(key).map(|_| EntityEntry {
                    name: key,
                    value: None,
                }));
            }
            let entity = match map.inner.entry(key.to_owned()) {
                Entry::Vacant(_) if keywords.contains(EntityKeyword::Remove) => return Ok(None),
                Entry::Vacant(entry) => entry.insert(Entity {
                    value: value.to_owned(),
                    published: keywords.contains(EntityKeyword::Publish),
                    description: description.unwrap_or_default(),
                }),
                Entry::Occupied(entry) if keywords.contains(EntityKeyword::Remove) => {
                    if entry.get().value == value {
                        entry.remove();
                        return Ok(Some(EntityEntry {
                            name: key,
                            value: None,
                        }));
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
                            return Ok(None);
                        }
                    } else {
                        entity.value.clear();
                        entity.value.push_str(value);
                        entity.apply_keywords(keywords);
                    }
                    entity
                }
            };
            Ok(Some(EntityEntry {
                name: key,
                value: Some(entity),
            }))
        }
        inner(self, key, value, description, keywords.into())
    }

    fn get(&self, key: &str) -> Option<&str> {
        if let Some(global) = self.globals.get(key) {
            return Some(global);
        }
        Some(&self.inner.get(key)?.value)
    }

    pub(crate) fn decode_entity(&self, key: &str) -> crate::Result<Option<&str>> {
        let Some(code) = key.strip_prefix('#') else {
            return Ok(self.get(key));
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

    #[test]
    fn set_new() {
        let mut map = EntityMap::new();
        map.set("key", "value", None, None).ok();
        assert_eq!(map.get("key"), Some("value"));
    }

    #[test]
    fn set_delete() {
        let mut map = EntityMap::new();
        map.set("key", "value", None, None).ok();
        map.set("key", "", None, EntityKeyword::Delete).ok();
        assert_eq!(map.get("key"), None);
    }

    #[test]
    fn set_replace() {
        let mut map = EntityMap::new();
        map.set("key", "value", Some("desc1".to_owned()), None).ok();
        map.set("key", "", Some("desc2".to_owned()), EntityKeyword::Publish)
            .ok();
        assert_eq!(
            map.inner.get("key"),
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
        map.set("key", "value1", None, EntityKeyword::Add).ok();
        map.set("key", "value2", None, EntityKeyword::Add).ok();
        map.set("key", "value3", None, EntityKeyword::Add).ok();
        map.set("key", "value2", None, EntityKeyword::Remove).ok();
        map.set("key", "x", None, EntityKeyword::Remove).ok();
        assert_eq!(map.get("key"), Some("value1|value3"));
    }

    #[test]
    fn protect_global() {
        let mut map = EntityMap::with_globals();
        let set_nonglobal = map.set("key", "value1", None, EntityKeyword::Add).is_ok();
        let set_global = map.set("amp", "value1", None, EntityKeyword::Add).is_ok();
        let remove_nonglobal = map.remove("key").is_ok();
        let remove_global = map.remove("amp").is_ok();
        assert_eq!(
            (set_nonglobal, set_global, remove_nonglobal, remove_global),
            (true, false, true, false)
        );
    }

    #[test]
    fn decode_entity_matched() {
        let mut map = EntityMap::new();
        map.set("key1", "value1", None, None).ok();
        map.set("key2", "value2", None, None).ok();
        assert_eq!(map.decode_entity("key1"), Ok(Some("value1")));
    }

    #[test]
    fn decode_entity_unmatched() {
        let mut map = EntityMap::new();
        map.set("key2", "value2", None, None).ok();
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
