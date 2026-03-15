use std::collections::HashMap;
use std::sync::LazyLock;

use super::visibility::EntityVisibility;

/// Stores information from the MUD (MUD variables). Once an entity is defined, an entity's value
/// can be referenced by using the `&Name;` syntax.
///
/// See [MXP specification: Entities](https://www.zuggsoft.com/zmud/mxp.htm#ENTITY).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entity {
    /// Value stored in the variable.
    pub value: String,
    /// Visibility to the client.
    pub visibility: EntityVisibility,
    /// Longer description of the entity.
    pub description: String,
}

impl Entity {
    pub(super) fn globals() -> HashMap<&'static [u8], &'static str> {
        let mut globals = HashMap::with_capacity(html_escape::NAMED_ENTITIES.len() + 1);
        globals.extend(html_escape::NAMED_ENTITIES);
        globals.insert(b"text", "&text;");
        globals
    }

    /// Returns a global entity if one is defined for the given name.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(mxp::Entity::global("lt"), Some("<"));
    /// assert_eq!(mxp::Entity::global("HP"), None);
    /// ```
    pub fn global<S: AsRef<[u8]>>(name: S) -> Option<&'static str> {
        static GLOBALS: LazyLock<HashMap<&'static [u8], &'static str>> =
            LazyLock::new(Entity::globals);

        GLOBALS.get(name.as_ref()).copied()
    }

    /// Returns a global entity if one is defined for the given name. This function is slower than
    /// [`Entity::global`] because it performs a linear search, but it can be evaluated at compile
    /// time.
    ///
    /// # Examples
    ///
    /// ```
    /// const LESS_THAN: &str = mxp::Entity::const_global("lt").unwrap();
    /// assert_eq!(LESS_THAN, "<");
    /// ```
    pub const fn const_global(name: &str) -> Option<&'static str> {
        const fn const_eq(mut a: &[u8], mut b: &[u8]) -> bool {
            if a.len() != b.len() {
                return false;
            }

            while let ([first_a, rest_a @ ..], [first_b, rest_b @ ..]) = (a, b) {
                if *first_a == *first_b {
                    a = rest_a;
                    b = rest_b;
                } else {
                    return false;
                }
            }

            true
        }

        let name = name.as_bytes();
        let mut i = 0;
        while i < html_escape::NAMED_ENTITIES.len() {
            let (key, value) = html_escape::NAMED_ENTITIES[i];
            if const_eq(key, name) {
                return Some(value);
            }
            i += 1;
        }
        None
    }

    /// Constructs new `Entity` with [`EntityVisibility::Default`] visibility and no description.
    pub const fn new(value: String) -> Self {
        Self {
            value,
            visibility: EntityVisibility::Default,
            description: String::new(),
        }
    }

    /// Returns `true` if `self.visibility` is [`EntityVisibility::Private`].
    pub const fn is_private(&self) -> bool {
        matches!(self.visibility, EntityVisibility::Private)
    }

    /// Returns `true` if `self.visibility` is [`EntityVisibility::Publish`].
    pub const fn is_published(&self) -> bool {
        matches!(self.visibility, EntityVisibility::Publish)
    }

    /// Treating the current value as a list of values separated by `'|'`, appends the specified
    /// `value` to the list.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut entity = mxp::Entity::new("1".to_owned());
    /// entity.add("2");
    /// entity.add("");
    /// entity.add("3");
    /// assert_eq!(entity.value, "1|2||3");
    /// ```
    pub fn add(&mut self, value: &str) {
        self.value.reserve(value.len() + 1);
        self.value.push('|');
        self.value.push_str(value);
    }

    /// Treating the current value as a list of values separated by `'|'`, removes the specified
    /// `value` from the list.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut entity = mxp::Entity::new("1|2|3|2||4".to_owned());
    /// entity.remove("2");
    /// assert_eq!(entity.value, "1|3||4");
    /// ```
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
