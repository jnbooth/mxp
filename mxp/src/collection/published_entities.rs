use std::slice;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublishedEntity {
    pub name: String,
    pub desc: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublishedEntities {
    inner: Vec<PublishedEntity>,
}

impl Default for PublishedEntities {
    fn default() -> Self {
        Self::new()
    }
}

impl PublishedEntities {
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn insert(&mut self, name: String, desc: String) {
        match self.inner.binary_search_by(|entity| entity.name.cmp(&name)) {
            Ok(pos) => self.inner[pos].desc = desc,
            Err(pos) => self.inner.insert(pos, PublishedEntity { name, desc }),
        }
    }

    pub fn iter(&self) -> slice::Iter<PublishedEntity> {
        self.inner.iter()
    }

    pub fn remove(&mut self, name: &str) {
        if let Ok(pos) = self
            .inner
            .binary_search_by(|entity| entity.name.as_str().cmp(name))
        {
            self.inner.remove(pos);
        }
    }
}
