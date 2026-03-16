use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use super::decoded::DecodedEntity;
use super::entity::Entity;
use super::iter::EntityInfo;
use super::iter::PublishedIter;
use super::visibility::EntityVisibility;
use crate::Error;
use crate::keyword::{EntityKeyword, KeywordFilter};
use crate::parse::{Decoder, ErrorKind};
use crate::parsed::ParsedEntityDefinition;

#[derive(Debug)]
/// This struct is created by [`EntityMap::define`]. See its documentation for more.
pub struct EntityEntry<'a> {
    /// Borrowed entity value, or `None` if the entity was removed.
    pub value: Option<&'a str>,
    /// If `true`, entity's visibility is [`EntityVisibility::Publish`]. If `false`, entity's
    /// visibility is [`EntityVisibility::Default`].
    ///
    /// Generally speaking, entity entries should only be processed by the client if `publish` is
    /// `true`.
    pub publish: bool,
}

impl<'a> EntityEntry<'a> {
    pub(crate) fn new(entity: Option<Cow<'a, Entity>>) -> Option<Self> {
        let entity = entity?;
        if entity.is_private() {
            return None;
        }
        let value = match entity {
            Cow::Borrowed(entity) => Some(entity.value.as_str()),
            Cow::Owned(_) => None,
        };
        Some(Self {
            value,
            publish: entity.is_published(),
        })
    }
}

/// Stores all entities for the current environment, both MXP-defined entities (as [`Entity`]) and
/// global XML entities (as `&'static str`).
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
    /// use mxp::entity::EntityMap;
    ///
    /// let map = EntityMap::new();
    /// // global entities not recognized
    /// assert!(map.decode("lt").is_err());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a new map with all global XML entities predefined.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::entity::EntityMap;
    ///
    /// let map = EntityMap::with_globals();
    /// assert_eq!(map.decode("lt"), Ok("<".into()));
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
    /// use mxp::entity::EntityMap;
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
    /// use mxp::entity::EntityMap;
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
    /// use mxp::entity::EntityMap;
    ///
    /// let mut map = EntityMap::with_globals();
    /// map.insert("HP".to_owned(), "150".to_owned());
    /// map.clear();
    /// assert!(map.decode("HP").is_err()); // custom entities cleared
    /// assert_eq!(map.decode("lt"), Ok("<".into())); // global entities not cleared
    /// ```
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Decodes an entity by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::entity::EntityMap;
    ///
    /// let mut map = EntityMap::with_globals();
    ///
    /// // Decoding a global entity
    /// assert_eq!(map.decode("lt"), Ok("<".into()));
    ///
    /// // Decoding a custom entity
    /// assert!(map.decode("HP").is_err()); // Not defined yet
    /// map.insert("HP".to_owned(), "150".to_owned());
    /// assert_eq!(map.decode("HP"), Ok("150".into()));
    ///
    /// // Decoding a decimal character code
    /// assert_eq!(map.decode("#32"), Ok(' '.into()));
    ///
    /// // Decoding a hexadecimal character code
    /// assert_eq!(map.decode("#x20"), Ok(' '.into()));
    ///
    /// // Decoding an invalid character code
    /// assert!(map.decode("#xQ").is_err());
    /// ```
    pub fn decode(&self, name: &str) -> crate::Result<DecodedEntity<'_>> {
        self.decode_entity::<()>(name)
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the
    /// map. Returns an error if the key is associated with a global XML entity, since those cannot
    /// be changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::entity::EntityMap;
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
    /// use mxp::entity::EntityMap;
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

    /// An iterator visiting all entities which have been marked as PUBLISH by the server, in
    /// arbitrary order. The iterator element type is `&'a` [`EntityInfo`].
    pub fn published(&self) -> PublishedIter<'_> {
        self.inner.iter().filter_map(|(k, v)| {
            v.is_published().then_some(EntityInfo {
                name: k,
                description: &v.description,
                value: &v.value,
            })
        })
    }

    /// Returns the value of a custom MXP entity, or `None` if there is no entity with the specified
    /// name or the entity with the specified name was marked as PRIVATE
    /// ([`EntityVisibility::Private`]) by the server.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::entity::EntityMap;
    ///
    /// let mut map = EntityMap::with_globals();
    /// assert_eq!(map.get("HP"), None);
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

    /// Fails if `name` is a global entity, that is, if [`self.is_global(name)`](Self::is_global).
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::entity::EntityMap;
    ///
    /// let map = EntityMap::with_globals();
    /// assert!(map.guard_global("HP").is_ok());
    /// assert!(map.guard_global("lt").is_err());
    /// assert!(map.guard_global("#031").is_err());
    /// ```
    pub fn guard_global(&self, name: &str) -> crate::Result<()> {
        if self.is_global(name) {
            return Err(Error::new(name, ErrorKind::CannotRedefineGlobalEntity));
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
    /// use mxp::entity::EntityMap;
    ///
    /// let mut map = EntityMap::with_globals();
    /// assert_eq!(map.get("HP"), None);
    /// assert!(map.insert("HP".to_owned(), "150".to_owned()).is_ok());
    /// assert_eq!(map.get("HP"), Some("150"));
    /// assert!(map.insert("lt".to_owned(), "!".to_owned()).is_err()); // cannot modify global entity
    /// ```
    pub fn insert(&mut self, name: String, value: String) -> crate::Result<()> {
        self.guard_global(&name)?;
        self.inner.insert(name, value.into());
        Ok(())
    }

    /// Applies an MXP [`ParsedEntityDefinition`]'s name, value, description and keywords.
    /// Depending on the keywords provided, this may cause an entity to be inserted, removed,
    /// updated, or replaced.
    ///
    /// If a new entity is defined, returns an occupied entry referencing the entity in this map.
    /// If an entity is removed, returns a vacant entry, i.e. `entry.value == None`.
    /// Returns an error if the name is associated with a global XML entity, since those cannot be
    /// changed.
    ///
    /// See [MXP specification: `<!ENTITY>`](https://www.zuggsoft.com/zmud/mxp.htm#ENTITY).
    pub fn define<'a>(
        &'a mut self,
        definition: ParsedEntityDefinition,
    ) -> crate::Result<Option<Cow<'a, Entity>>> {
        let ParsedEntityDefinition {
            name,
            desc,
            value,
            keywords,
        } = definition;
        self.guard_global(name)?;
        let visibility = keywords.into();
        if keywords.contains(EntityKeyword::Delete) {
            let Some(mut entity) = self.inner.remove(name) else {
                return Ok(None);
            };
            if visibility != EntityVisibility::Default {
                entity.visibility = visibility;
            }
            return Ok(Some(Cow::Owned(entity)));
        }
        let entity = match self.inner.entry(name.to_owned()) {
            Entry::Vacant(_) if keywords.contains(EntityKeyword::Remove) => return Ok(None),
            Entry::Vacant(entry) => entry.insert(Entity {
                value: value.to_owned(),
                visibility,
                description: desc.unwrap_or_default().to_owned(),
            }),
            Entry::Occupied(entry) if keywords.contains(EntityKeyword::Remove) => {
                if entry.get().value == value {
                    let mut entity = entry.remove();
                    entity.value.clear();
                    if visibility != EntityVisibility::Default {
                        entity.visibility = visibility;
                    }
                    return Ok(Some(Cow::Owned(entity)));
                }
                let entity = entry.into_mut();
                entity.remove(value);
                entity
            }
            Entry::Occupied(entry) if keywords.contains(EntityKeyword::Add) => {
                let entity = entry.into_mut();
                entity.add(value);
                entity
            }
            Entry::Occupied(entry) => {
                let entity = entry.into_mut();
                value.clone_into(&mut entity.value);
                if let Some(desc) = desc {
                    desc.clone_into(&mut entity.description);
                }
                entity
            }
        };
        if visibility != EntityVisibility::Default {
            entity.visibility = visibility;
        }
        Ok(Some(Cow::Borrowed(entity)))
    }
}

impl Decoder for EntityMap {
    fn get_entity<K: KeywordFilter>(&self, name: &str) -> Option<&str> {
        if let Some(&global) = self.globals.get(name.as_bytes()) {
            return Some(global);
        }
        Some(self.inner.get(name)?.value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use flagset::FlagSet;

    use super::*;

    fn define<'a, T>(
        map: &'a mut EntityMap,
        name: &str,
        value: &str,
        desc: Option<&str>,
        keywords: T,
    ) -> crate::Result<Option<Cow<'a, Entity>>>
    where
        T: Into<FlagSet<EntityKeyword>>,
    {
        map.define(ParsedEntityDefinition {
            name,
            value,
            desc,
            keywords: keywords.into(),
        })
    }

    #[test]
    fn define_new() {
        let mut map = EntityMap::new();
        define(&mut map, "key", "value", None, None).ok();
        assert_eq!(map.get("key"), Some("value"));
    }

    #[test]
    fn define_delete() {
        let mut map = EntityMap::new();
        define(&mut map, "key", "value", None, None).ok();
        define(&mut map, "key", "", None, EntityKeyword::Delete).ok();
        assert_eq!(map.get("key"), None);
    }

    #[test]
    fn define_replace() {
        let mut map = EntityMap::new();
        define(&mut map, "key", "value", Some("desc1"), None).ok();
        define(&mut map, "key", "", Some("desc2"), EntityKeyword::Publish).ok();
        assert_eq!(
            map.inner.get("key"),
            Some(&Entity {
                value: String::new(),
                visibility: EntityVisibility::Publish,
                description: "desc2".to_owned()
            })
        );
    }

    #[test]
    fn define_add_and_remove() {
        let mut map = EntityMap::new();
        define(&mut map, "key", "value1", None, EntityKeyword::Add).ok();
        define(&mut map, "key", "value2", None, EntityKeyword::Add).ok();
        define(&mut map, "key", "value3", None, EntityKeyword::Add).ok();
        define(&mut map, "key", "value2", None, EntityKeyword::Remove).ok();
        define(&mut map, "key", "x", None, EntityKeyword::Remove).ok();
        assert_eq!(map.get("key"), Some("value1|value3"));
    }

    #[test]
    fn protect_global() {
        let mut map = EntityMap::with_globals();
        let set_nonglobal = define(&mut map, "key", "value1", None, EntityKeyword::Add).is_ok();
        let set_global = define(&mut map, "amp", "value1", None, EntityKeyword::Add).is_ok();
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
        define(&mut map, "key1", "value1", None, None).ok();
        define(&mut map, "key2", "value2", None, None).ok();
        assert_eq!(map.decode("key1"), Ok("value1".into()));
    }

    #[test]
    fn decode_entity_unmatched() {
        let mut map = EntityMap::new();
        define(&mut map, "key2", "value2", None, None).ok();
        assert!(map.decode("key1").is_err());
    }

    #[test]
    fn decode_decimal() {
        assert_eq!(EntityMap::new().decode("#32"), Ok('\x20'.into()));
    }

    #[test]
    fn decode_hex() {
        assert_eq!(EntityMap::new().decode("#x7E"), Ok('\x7e'.into()));
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
        assert_eq!(EntityMap::new().decode("#10"), Ok("".into()));
    }

    #[test]
    fn decode_above_range() {
        assert_eq!(EntityMap::new().decode("#x90"), Ok("".into()));
    }
}
