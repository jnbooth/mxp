use super::entity::Entity;
use std::collections::hash_map;
use std::iter::FusedIterator;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityInfo<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub value: &'a str,
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct PublishedIter<'a> {
    pub(super) inner: hash_map::Iter<'a, String, Entity>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::EntityMap;
    use crate::keyword::EntityKeyword;
    use enumeration::EnumSet;

    #[test]
    fn published_iter() {
        let mut map = EntityMap::new();
        let published = enums![EntityKeyword::Publish];
        let unpublished = EnumSet::new();
        map.set("key1", "val1", Some("desc1".to_owned()), unpublished);
        map.set("key2", "val2", Some("desc2".to_owned()), published);
        map.set("key3", "val3", Some("desc3".to_owned()), published);
        map.set("key4", "val4", Some("desc4".to_owned()), unpublished);
        map.set("key5", "val5", None, published);
        let mut published = map.published().collect::<Vec<_>>();
        published.sort();
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
