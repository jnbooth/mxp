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
