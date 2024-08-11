use casefold::ascii::{CaseFold, CaseFoldMap};
use std::collections::hash_map;
use std::sync::OnceLock;

pub struct Lookup<T> {
    inner: OnceLock<CaseFoldMap<&'static str, T>>,
    init: fn() -> Vec<(&'static str, T)>,
}

impl<T> Lookup<T> {
    pub const fn new(init: fn() -> Vec<(&'static str, T)>) -> Self {
        Self {
            inner: OnceLock::new(),
            init,
        }
    }

    pub fn get(&self, name: &str) -> Option<&T> {
        self.get_or_init().get(name)
    }

    pub fn values(&self) -> hash_map::Values<CaseFold<&'static str>, T> {
        self.get_or_init().values()
    }

    fn get_or_init(&self) -> &CaseFoldMap<&'static str, T> {
        if let Some(map) = self.inner.get() {
            return map;
        };
        self.initialize()
    }

    #[cold]
    fn initialize(&self) -> &CaseFoldMap<&'static str, T> {
        let map = (self.init)()
            .into_iter()
            .map(|(k, v)| (CaseFold::new(k), v))
            .collect();
        self.inner.set(map).ok().unwrap();
        self.inner.get().unwrap()
    }
}
