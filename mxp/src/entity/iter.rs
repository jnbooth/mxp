use std::collections::hash_map;
use std::iter;

use super::entity::Entity;

/// This struct is created by [`EntityMap::published`](crate::entity::EntityMap::published).
/// See its documentation for more.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityInfo<'a> {
    /// Entity name.
    pub name: &'a str,
    /// Entity description.
    pub description: &'a str,
    /// Current value of the entity, as defined by the server.
    pub value: &'a str,
}

/// Type alias for the iterator returned by [`EntityMap::published`](crate::entity::EntityMap::published).
pub type PublishedIter<'a> = iter::FilterMap<
    hash_map::Iter<'a, String, Entity>,
    fn((&'a String, &'a Entity)) -> Option<EntityInfo<'a>>,
>;
