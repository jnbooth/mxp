use std::collections::hash_map;
use std::iter;

use super::entity::Entity;

/// Information about a published entity.
///
/// See [MXP specification: Entities](https://www.zuggsoft.com/zmud/mxp.htm#ENTITY).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityInfo<'a> {
    /// Entity name.
    pub name: &'a str,
    /// Entity description.
    pub description: &'a str,
    /// Current value of the entity, as defined by the server.
    pub value: &'a str,
}

/// Type alias for the iterator returned by [`EntityMap::published`](crate::EntityMap::published).
pub type PublishedIter<'a> = iter::FilterMap<
    hash_map::Iter<'a, String, Entity>,
    fn((&'a String, &'a Entity)) -> Option<EntityInfo<'a>>,
>;
