use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use uncased::{Uncased, UncasedStr};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CaseFoldMap<V> {
    inner: HashMap<Uncased<'static>, V>,
}

impl<V> CaseFoldMap<V> {
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

    pub fn insert(&mut self, key: String, value: V) -> Option<V> {
        self.inner.insert(Uncased::from_owned(key), value)
    }

    pub fn remove(&mut self, key: &str) -> Option<V> {
        self.inner.remove(UncasedStr::new(key))
    }
}

impl<V> Deref for CaseFoldMap<V> {
    type Target = HashMap<Uncased<'static>, V>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<V> DerefMut for CaseFoldMap<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<V> Extend<(String, V)> for CaseFoldMap<V> {
    fn extend<T: IntoIterator<Item = (String, V)>>(&mut self, iter: T) {
        self.inner.extend(
            iter.into_iter()
                .map(|(key, value)| (Uncased::from_owned(key), value)),
        );
    }
}

impl<V> FromIterator<(String, V)> for CaseFoldMap<V> {
    fn from_iter<T: IntoIterator<Item = (String, V)>>(iter: T) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}
