use std::borrow::Cow;
use std::collections::{HashMap, hash_map};
use std::hash::{BuildHasher, RandomState};
use std::ops::{Deref, DerefMut};
use std::{fmt, iter};

use uncased::{Uncased, UncasedStr};

#[derive(Default)]
pub(crate) struct CaseFoldMap<'a, V, S = RandomState> {
    base: HashMap<Uncased<'a>, V, S>,
}

impl<V, S> PartialEq for CaseFoldMap<'_, V, S>
where
    V: PartialEq,
    S: BuildHasher,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base
    }
}

impl<V, S> Eq for CaseFoldMap<'_, V, S>
where
    V: Eq,
    S: BuildHasher,
{
}

impl<V, S> Clone for CaseFoldMap<'_, V, S>
where
    V: Clone,
    S: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.base.clone_from(&source.base);
    }
}

impl<V: fmt::Debug, S> fmt::Debug for CaseFoldMap<'_, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.base.fmt(f)
    }
}

impl<V> CaseFoldMap<'_, V> {
    pub fn new() -> Self {
        Self {
            base: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            base: HashMap::with_capacity(capacity),
        }
    }
}

impl<'a, V, S: BuildHasher> CaseFoldMap<'a, V, S> {
    pub fn contains_key(&self, key: &str) -> bool {
        self.base.contains_key(UncasedStr::new(key))
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.base.get(UncasedStr::new(key))
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        self.base.get_mut(UncasedStr::new(key))
    }

    pub fn insert<K: Into<Uncased<'a>>>(&mut self, key: K, value: V) -> Option<V> {
        self.base.insert(key.into(), value)
    }

    pub fn remove(&mut self, key: &str) -> Option<V> {
        self.base.remove(UncasedStr::new(key))
    }
}

impl<'a, V, S> Deref for CaseFoldMap<'a, V, S> {
    type Target = HashMap<Uncased<'a>, V, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<V, S> DerefMut for CaseFoldMap<'_, V, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<'a, K, V, S> Extend<(K, V)> for CaseFoldMap<'a, V, S>
where
    K: Into<Cow<'a, str>>,
    S: BuildHasher,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        self.base.extend(
            iter.into_iter()
                .map(|(key, value)| (Uncased::new(key), value)),
        );
    }
}

impl<'a, K, V> FromIterator<(K, V)> for CaseFoldMap<'a, V>
where
    K: Into<Cow<'a, str>>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

impl<'a, V, S> IntoIterator for &'a CaseFoldMap<'a, V, S> {
    type Item = (&'a str, &'a V);

    type IntoIter = iter::Map<
        hash_map::Iter<'a, Uncased<'a>, V>,
        fn((&'a Uncased<'a>, &'a V)) -> (&'a str, &'a V),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.base.iter().map(|(k, v)| (k.as_str(), v))
    }
}

impl<'a, V, S> IntoIterator for CaseFoldMap<'a, V, S> {
    type Item = (Uncased<'a>, V);

    type IntoIter = hash_map::IntoIter<Uncased<'a>, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.base.into_iter()
    }
}
