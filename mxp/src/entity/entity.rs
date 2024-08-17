use enumeration::EnumSet;

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

    pub fn apply_keywords(&mut self, keywords: EnumSet<EntityKeyword>) {
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
