use std::collections::HashMap;
use std::collections::hash_map::Entry;

use flagset::FlagSet;

use super::decoded::DecodedEntity;
use super::entity::Entity;
use super::iter::PublishedIter;
use crate::keyword::{EntityKeyword, KeywordFilter};
use crate::parse::{Decoder, ErrorKind};

/// Entry in an [`EntityMap`].
pub struct EntityEntry<'a> {
    /// Entity name.
    pub name: &'a str,
    /// Entity value, or `None` if no entity matching `name` exists in the map.
    pub value: Option<&'a Entity>,
}

/// Stores all entities for the current environment, both MXP-defined entities ([`Entity`]) and
/// global XML entities (static string slices).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityMap {
    inner: HashMap<String, Entity>,
    globals: HashMap<&'static [u8], &'static str>,
}

impl EntityMap {
    /// Constructs a new map with no predefined entities.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::EntityMap;
    ///
    /// let map = EntityMap::new();
    /// assert_eq!(map.decode("lt"), Ok(None));
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a new map with all global XML entities predefined.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::EntityMap;
    ///
    /// let map = EntityMap::with_globals();
    /// assert_eq!(map.decode("lt"), Ok(Some("<".into())));
    /// ```
    pub fn with_globals() -> Self {
        Self {
            inner: HashMap::new(),
            globals: Entity::globals(),
        }
    }

    /// Returns `true` if the map contains no custom entities.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::EntityMap;
    ///
    /// let mut map = EntityMap::with_globals();
    /// assert!(map.is_empty());
    /// map.insert("HP".to_owned(), "150".to_owned());
    /// assert!(!map.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of custom entities in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::EntityMap;
    ///
    /// let mut map = EntityMap::with_globals();
    /// assert!(map.is_empty());
    /// map.insert("HP".to_owned(), "150".to_owned());
    /// map.insert("MP".to_owned(), "100".to_owned());
    /// map.insert("HP".to_owned(), "125".to_owned());
    /// assert_eq!(map.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Clears all custom entities.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::EntityMap;
    ///
    /// let mut map = EntityMap::with_globals();
    /// map.insert("HP".to_owned(), "150".to_owned());
    /// map.clear();
    /// assert_eq!(map.decode("HP"), Ok(None)); // custom entities cleared
    /// assert_eq!(map.decode("lt"), Ok(Some("<".into()))); // global entities not cleared
    /// ```
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Decodes an entity by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::EntityMap;
    ///
    /// let mut map = EntityMap::with_globals();
    ///
    /// // Decoding a global entity
    /// assert_eq!(map.decode("lt"), Ok(Some("<".into())));
    ///
    /// // Decoding a custom entity
    /// assert_eq!(map.decode("HP"), Ok(None));
    /// map.insert("HP".to_owned(), "150".to_owned());
    /// assert_eq!(map.decode("HP"), Ok(Some("150".into())));
    ///
    /// // Decoding a decimal character code
    /// assert_eq!(map.decode("#32"), Ok(Some(' '.into())));
    ///
    /// // Decoding a hexadecimal character code
    /// assert_eq!(map.decode("#x20"), Ok(Some(' '.into())));
    ///
    /// // Decoding an invalid character code
    /// assert!(map.decode("#xQ").is_err());
    /// ```
    pub fn decode(&self, name: &str) -> crate::Result<Option<DecodedEntity<'_>>> {
        self.decode_entity::<()>(name)
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the
    /// map. Returns an error if the key is associated with a global XML entity, since those cannot
    /// be changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::EntityMap;
    ///
    /// let mut map = EntityMap::with_globals();
    /// map.insert("HP".to_owned(), "150".to_owned());
    /// assert!(map.remove("HP").unwrap().is_some());
    /// assert!(map.remove("HP").unwrap().is_none());
    /// assert!(map.remove("lt").is_err()); // cannot modify global entity
    /// ```
    pub fn remove(&mut self, name: &str) -> crate::Result<Option<Entity>> {
        self.guard_global(name)?;
        Ok(self.inner.remove(name))
    }

    /// Returns `true` if there is a global XML entity associated with the specified name.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::EntityMap;
    ///
    /// let mut map = EntityMap::with_globals();
    /// assert!(map.is_global("lt"));
    /// assert!(map.is_global("#031"));
    /// map.insert("HP".to_owned(), "150".to_owned());
    /// assert!(!map.is_global("HP"));
    /// ```
    pub fn is_global(&self, name: &str) -> bool {
        name.starts_with('#') || self.globals.contains_key(name.as_bytes())
    }

    /// Iterates through all entities which have been marked as PUBLISH by the server, in the form
    /// of [`EntityInfo`](super::EntityInfo) entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::{EntityInfo, EntityKeyword, EntityMap};
    ///
    /// let mut map = EntityMap::new();
    /// map.set("k1", "v1", Some("desc1".to_owned()), None).unwrap();
    /// map.set("k2", "v2", Some("desc2".to_owned()), EntityKeyword::Publish)
    ///     .unwrap();
    /// map.set("k3", "v3", Some("desc3".to_owned()), EntityKeyword::Publish)
    ///     .unwrap();
    /// map.set("k4", "v4", Some("desc4".to_owned()), None).unwrap();
    /// map.set("k5", "v5", None, EntityKeyword::Publish).unwrap();
    ///
    /// let mut published = map.published().collect::<Vec<_>>();
    /// published.sort_unstable_by_key(|i| i.name);
    /// assert_eq!(
    ///     published,
    ///     &[
    ///         EntityInfo {
    ///             name: "k2",
    ///             value: "v2",
    ///             description: "desc2",
    ///         },
    ///         EntityInfo {
    ///             name: "k3",
    ///             value: "v3",
    ///             description: "desc3",
    ///         },
    ///         EntityInfo {
    ///             name: "k5",
    ///             value: "v5",
    ///             description: "",
    ///         },
    ///     ]
    /// );
    /// ```
    pub fn published(&self) -> PublishedIter<'_> {
        self.inner.iter().filter_map(|(k, v)| {
            v.is_published().then_some(super::EntityInfo {
                name: k,
                description: &v.description,
                value: &v.value,
            })
        })
    }

    /// Returns the value of a custom MXP entity, or `None` if there is no entity with the specified
    /// name or the entity with the specified name was marked as PRIVATE by the server.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::EntityMap;
    ///
    /// let mut map = EntityMap::with_globals();
    /// assert_eq!(map.decode("HP"), Ok(None));
    /// map.insert("HP".to_owned(), "150".to_owned());
    /// assert_eq!(map.get("HP"), Some("150"));
    /// assert_eq!(map.get("lt"), None); // global entities are not queried
    /// ```
    pub fn get(&self, name: &str) -> Option<&str> {
        let entity = self.inner.get(name)?;
        if entity.is_private() {
            return None;
        }
        Some(entity.value.as_str())
    }

    /// Fails if `name` is a global entity, that is, if `self.is_global(name)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::EntityMap;
    ///
    /// let map = EntityMap::with_globals();
    /// assert!(map.guard_global("HP").is_ok());
    /// assert!(map.guard_global("lt").is_err());
    /// assert!(map.guard_global("#031").is_err());
    /// ```
    pub fn guard_global(&self, name: &str) -> crate::Result<()> {
        if self.is_global(name) {
            return Err(crate::Error::new(name, ErrorKind::CannotRedefineEntity));
        }
        Ok(())
    }

    /// Inserts a custom entity with the specified name and value. Returns `false` if the specified
    /// name is already associated with a global XML entity. Otherwise, performs the insertion and
    /// returns `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::EntityMap;
    ///
    /// let mut map = EntityMap::with_globals();
    /// assert_eq!(map.get("HP"), None);
    /// assert!(map.insert("HP".to_owned(), "150".to_owned()));
    /// assert_eq!(map.get("HP"), Some("150"));
    /// assert!(!map.insert("lt".to_owned(), "!".to_owned())); // cannot modify global entity
    /// ```
    pub fn insert(&mut self, name: String, value: String) -> bool {
        if self.globals.contains_key(name.as_bytes()) {
            return false;
        }
        self.inner.insert(name, value.into());
        true
    }

    /// Applies an MXP entity definition with the specified name, value, description and keywords,
    /// as provided by `<!ENTITY>` declarations from the server.
    /// Depending on the keywords provided, this may cause an entity to be inserted, removed,
    /// updated, or replaced.
    ///
    /// If a new entity is defined, returns an occupied entry referencing the entity in this map.
    /// If an entity is removed, returns a vacant entry, i.e. `entry.value == None`.
    /// Returns an error if the name is associated with a global XML entity, since those cannot be
    /// changed.
    ///
    /// See [MXP specification: `<!ENTITY>`](https://www.zuggsoft.com/zmud/mxp.htm#ENTITY).
    /// ```
    pub fn set<'a, T: Into<FlagSet<EntityKeyword>>>(
        &'a mut self,
        name: &'a str,
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
                    visibility: keywords.into(),
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
                        let old_visibility = entity.visibility;
                        entity.apply_keywords(keywords);
                        if entity.visibility == old_visibility {
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
        inner(self, name, value, description, keywords.into())
    }
}

impl Decoder for EntityMap {
    fn get_entity<F>(&self, name: &str) -> Option<&str>
    where
        F: KeywordFilter,
    {
        if let Some(&global) = self.globals.get(name.as_bytes()) {
            return Some(global);
        }
        Some(self.inner.get(name)?.value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntityVisibility, Error};

    #[test]
    fn set_new() {
        let mut map = EntityMap::new();
        map.set("key", "value", None, None).ok();
        assert_eq!(map.decode("key"), Ok(Some("value".into())));
    }

    #[test]
    fn set_delete() {
        let mut map = EntityMap::new();
        map.set("key", "value", None, None).ok();
        map.set("key", "", None, EntityKeyword::Delete).ok();
        assert_eq!(map.decode("key"), Ok(None));
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
                visibility: EntityVisibility::Published,
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
        assert_eq!(map.decode("key"), Ok(Some("value1|value3".into())));
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
        assert_eq!(map.decode("key1"), Ok(Some("value1".into())));
    }

    #[test]
    fn decode_entity_unmatched() {
        let mut map = EntityMap::new();
        map.set("key2", "value2", None, None).ok();
        assert_eq!(map.decode("key1"), Ok(None));
    }

    #[test]
    fn decode_decimal() {
        assert_eq!(EntityMap::new().decode("#32"), Ok(Some('\x20'.into())));
    }

    #[test]
    fn decode_hex() {
        assert_eq!(EntityMap::new().decode("#x7E"), Ok(Some('\x7e'.into())));
    }

    #[test]
    fn decode_invalid_number() {
        assert_eq!(
            EntityMap::new().decode("#x7z"),
            Err(Error::new("#x7z", ErrorKind::InvalidEntityNumber))
        );
    }

    #[test]
    fn decode_below_range() {
        assert_eq!(
            EntityMap::new().decode("#10"),
            Err(Error::new("#10", ErrorKind::DisallowedEntityNumber))
        );
    }

    #[test]
    fn decode_above_range() {
        assert_eq!(
            EntityMap::new().decode("#x90"),
            Err(Error::new("#x90", ErrorKind::DisallowedEntityNumber))
        );
    }
}
