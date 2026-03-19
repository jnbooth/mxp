use std::fmt;
use std::str::FromStr;

use crate::arguments::Arguments;
use crate::parse::Words;
use crate::{CaseFoldMap, Error, ErrorKind, validate};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Attribute {
    position: usize,
    value: Option<String>,
}

// Note: this count may be incorrect if the arguments are poorly formed (.e.g "==="), but that's
// fine because the list won't be completely constructed in that case anyway.
fn count_args(iter: Words) -> usize {
    let (count, _) = iter.fold((0, false), |(count, in_named), word| {
        if in_named {
            return (count, false);
        }
        if word == "=" {
            return (count, true);
        }
        (count + 1, false)
    });
    count
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AttributeList {
    attributes: CaseFoldMap<'static, Attribute>,
}

impl AttributeList {
    /// Constructs a new, empty attribute list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the list contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut attrs = mxp::AttributeList::new();
    /// assert!(attrs.is_empty());
    /// attrs.push("fore".into(), Some("red".into()));
    /// assert!(!attrs.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    /// Returns the number of elements in the list.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut attrs = mxp::AttributeList::new();
    /// assert_eq!(attrs.len(), 0);
    /// attrs.push("fore".into(), Some("red".into()));
    /// assert_eq!(attrs.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// Returns `true` if there is an attribute with the specified name.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut attrs = mxp::AttributeList::new();
    /// assert!(!attrs.contains("fore"));
    /// attrs.push("fore".into(), Some("red".into()));
    /// assert!(attrs.contains("fore"));
    pub fn contains(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// Returns the default value associated with an attribute by name.
    /// Returns `None` if there is no attribute with that name, or if the attribute has no default
    /// value.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut attrs = mxp::AttributeList::new();
    /// attrs.push("fore".into(), Some("red".into()));
    /// assert_eq!(attrs.get("fore"), Some("red"));
    pub fn get(&self, name: &str) -> Option<&str> {
        Some(self.attributes.get(name)?.value.as_ref()?.as_str())
    }

    /// Returns the position of the attribute in the list that matches the specified name.
    /// Returns `None` if there is no attribute with that name.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut attrs = mxp::AttributeList::new();
    /// attrs.push("fore".into(), Some("red".into()));
    /// attrs.push("back".into(), None);
    /// assert_eq!(attrs.position("fore"), Some(0));
    /// assert_eq!(attrs.position("back"), Some(1));
    pub fn position(&self, name: &str) -> Option<usize> {
        Some(self.attributes.get(name)?.position)
    }

    /// Appends an attribute to the end of the list, optionally with a default value. The
    /// attribute's [`position`](Self::position`) will be equal to the [`len`] beforehand.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut attrs = mxp::AttributeList::new();
    /// attrs.push("fore".into(), Some("red".into())); // default value is "red"
    /// attrs.push("back".into(), None); // no default value
    /// attrs.push("size".into(), Some(String::new())); // default value is an empty string
    pub fn push(&mut self, name: String, default_value: Option<String>) {
        self.attributes.insert(
            name,
            Attribute {
                position: self.attributes.len(),
                value: default_value,
            },
        );
    }

    /// Finds the value of an entity, using an element's attribute list to identify arguments
    /// and provide default values.
    pub(crate) fn find<'a>(&'a self, name: &str, args: &Arguments<'a>) -> Option<&'a str> {
        if let Some(entity) = args.get(name) {
            return Some(entity);
        }
        let attribute = self.attributes.get(name)?;
        if let Some(entity) = args.at(attribute.position) {
            return Some(entity);
        }
        attribute.value.as_deref()
    }

    /// Adds attributes to the list from parsed arguments.
    pub(crate) fn append(&mut self, iter: Words) -> crate::Result<()> {
        let size = count_args(iter.clone());
        self.attributes.reserve(size);
        for entry in iter.args() {
            let (name, value) = entry?;
            validate(name, ErrorKind::InvalidArgumentName)?;
            if self.attributes.contains_key(name) {
                return Err(Error::new(name, ErrorKind::DuplicateAttributeInAttlist));
            }
            self.push(name.to_owned(), value.map(ToOwned::to_owned));
        }
        Ok(())
    }
}

impl fmt::Display for AttributeList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::display::MaybeQuote;

        for i in 0..self.attributes.len() {
            if i != 0 {
                f.write_str(" ")?;
            }
            let Some((name, attr)) = self.attributes.iter().find(|(_, v)| v.position == i) else {
                continue;
            };
            match &attr.value {
                Some(value) => write!(f, "{name}={}", MaybeQuote(value))?,
                None => write!(f, "{name}")?,
            }
        }
        Ok(())
    }
}

impl TryFrom<Words<'_>> for AttributeList {
    type Error = Error;

    fn try_from(value: Words) -> crate::Result<Self> {
        let mut list = AttributeList::new();
        list.append(value)?;
        Ok(list)
    }
}

impl FromStr for AttributeList {
    type Err = Error;

    fn from_str(s: &str) -> crate::Result<Self> {
        Words::new(s).try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_count() {
        let words = Words::new("EL RName '<FONT COLOR=Red><B>' FLAG=\"RoomName\"");
        assert_eq!(count_args(words), 4);
    }

    #[test]
    fn fmt() {
        let words = Words::new("EL RName FLAG=\"Room Name\" Owner Locked=0");
        let formatted = AttributeList::try_from(words).unwrap().to_string();
        assert_eq!(formatted, "EL RName FLAG=\"Room Name\" Owner Locked=0");
    }
}
