use std::collections::HashMap;
use std::sync::LazyLock;

use flagset::FlagSet;

use super::visibility::EntityVisibility;
use crate::EntityKeyword;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entity {
    pub value: String,
    pub visibility: EntityVisibility,
    pub description: String,
}

impl Entity {
    pub(super) fn globals() -> HashMap<&'static [u8], &'static str> {
        html_escape::NAMED_ENTITIES.iter().copied().collect()
    }

    pub fn global(name: &str) -> Option<&'static str> {
        static GLOBALS: LazyLock<HashMap<&'static [u8], &'static str>> =
            LazyLock::new(Entity::globals);

        GLOBALS.get(name.as_bytes()).copied()
    }

    pub const fn new(value: String) -> Self {
        Self {
            value,
            visibility: EntityVisibility::Default,
            description: String::new(),
        }
    }

    pub const fn is_private(&self) -> bool {
        matches!(self.visibility, EntityVisibility::Private)
    }

    pub const fn is_published(&self) -> bool {
        matches!(self.visibility, EntityVisibility::Published)
    }

    pub fn apply_keywords<T: Into<FlagSet<EntityKeyword>>>(&mut self, keywords: T) {
        let keywords = keywords.into();
        if keywords.contains(EntityKeyword::Private) {
            self.visibility = EntityVisibility::Private;
        } else if keywords.contains(EntityKeyword::Publish) {
            self.visibility = EntityVisibility::Published;
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

impl From<String> for Entity {
    fn from(value: String) -> Self {
        Self::new(value)
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
        assert_eq!(entity.visibility, EntityVisibility::Published);
    }

    #[test]
    fn apply_private() {
        let mut entity = Entity::new(String::new());
        entity.apply_keywords(EntityKeyword::Publish);
        entity.apply_keywords(None);
        entity.apply_keywords(EntityKeyword::Private);
        entity.apply_keywords(None);
        assert_eq!(entity.visibility, EntityVisibility::Private);
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
