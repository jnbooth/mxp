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

#[cfg(test)]
mod tests {
    use flagset::FlagSet;

    use super::*;
    use crate::entity::EntityMap;
    use crate::keyword::EntityKeyword;

    #[test]
    fn published_iter() {
        let mut map = EntityMap::new();
        let published = FlagSet::from(EntityKeyword::Publish);
        let unpublished = FlagSet::default();
        map.set("key1", "val1", Some("desc1".to_owned()), unpublished)
            .ok();
        map.set("key2", "val2", Some("desc2".to_owned()), published)
            .ok();
        map.set("key3", "val3", Some("desc3".to_owned()), published)
            .ok();
        map.set("key4", "val4", Some("desc4".to_owned()), unpublished)
            .ok();
        map.set("key5", "val5", None, published).ok();
        let mut published = map.published().collect::<Vec<_>>();
        published.sort_unstable_by_key(|i| i.name);
        let expected = vec![
            EntityInfo {
                name: "key2",
                value: "val2",
                description: "desc2",
            },
            EntityInfo {
                name: "key3",
                value: "val3",
                description: "desc3",
            },
            EntityInfo {
                name: "key5",
                value: "val5",
                description: "",
            },
        ];
        assert_eq!(published, expected);
    }
}
