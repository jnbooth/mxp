use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use uncased::{Uncased, UncasedStr};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CaseFoldMap<'a, V> {
    inner: HashMap<Uncased<'a>, V>,
}

impl<'a, V> CaseFoldMap<'a, V> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.inner.get(UncasedStr::new(key))
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        self.inner.get_mut(UncasedStr::new(key))
    }

    pub fn insert<K: Into<Uncased<'a>>>(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key.into(), value)
    }

    pub fn remove(&mut self, key: &str) -> Option<V> {
        self.inner.remove(UncasedStr::new(key))
    }
}

impl<'a, V> Deref for CaseFoldMap<'a, V> {
    type Target = HashMap<Uncased<'a>, V>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<V> DerefMut for CaseFoldMap<'_, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, K, V> Extend<(K, V)> for CaseFoldMap<'a, V>
where
    K: Into<Cow<'a, str>>,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        self.inner.extend(
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
