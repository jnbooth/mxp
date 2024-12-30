use flagset::FlagSet;

use crate::EntityKeyword;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    pub value: String,
    pub published: bool,
    pub description: String,
}

impl Entity {
    pub const fn new(value: String) -> Self {
        Self {
            value,
            published: false,
            description: String::new(),
        }
    }

    pub fn apply_keywords<T: Into<FlagSet<EntityKeyword>>>(&mut self, keywords: T) {
        let keywords = keywords.into();
        if keywords.contains(EntityKeyword::Private) {
            self.published = false;
        } else if keywords.contains(EntityKeyword::Publish) {
            self.published = true;
        }
    }

    pub fn add(&mut self, value: &str) {
        self.value.reserve(value.len() + 1);
        self.value.push('|');
        self.value.push_str(value);
    }

    pub fn remove(&mut self, value: &str) {
        self.value = self
            .value
            .split('|')
            .filter(|item| *item != value)
            .collect::<Vec<_>>()
            .join("|");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_publish() {
        let mut entity = Entity::new(String::new());
        entity.apply_keywords(EntityKeyword::Publish);
        entity.apply_keywords(None);
        assert!(entity.published);
    }

    #[test]
    fn apply_private() {
        let mut entity = Entity::new(String::new());
        entity.apply_keywords(EntityKeyword::Publish);
        entity.apply_keywords(None);
        entity.apply_keywords(EntityKeyword::Private);
        entity.apply_keywords(None);
        assert!(!entity.published);
    }

    #[test]
    fn add_and_remove() {
        let mut entity = Entity::new("1".to_owned());
        entity.add("2");
        entity.add("3");
        entity.add("");
        entity.add("2");
        entity.add("3");
        entity.remove("2");
        assert_eq!(entity.value, "1|3||3");
    }
}
