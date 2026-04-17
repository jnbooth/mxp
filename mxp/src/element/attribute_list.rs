use std::fmt;
use std::str::FromStr;

use crate::arguments::Arguments;
use crate::parse::ArgumentParser;
use crate::{CaseFoldMap, Error, ErrorKind, validate};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Attribute {
    position: usize,
    value: Option<String>,
}

/// Predefined attributes belonging to an [`Element`](crate::Element).
///
/// Elements can define attributes during their own definition (`<!ELEMENT...>`) or in an attribute
/// list definition (`<!ATTLIST...>`).
///
/// See [MXP specification: Attributes](https://www.zuggsoft.com/zmud/mxp.htm#ATTLIST).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct AttributeList {
    attributes: CaseFoldMap<'static, Attribute>,
}

impl Clone for AttributeList {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            attributes: self.attributes.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.attributes.clone_from(&source.attributes);
    }
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
    /// attribute's [`position`](Self::position) will be equal to the [`len`](Self::len) beforehand.
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

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `AttributeList`. The collection may reserve more space to speculatively
    /// avoid frequent reallocations. After calling `reserve`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows [`usize`].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut attrs = mxp::AttributeList::new();
    /// attrs.reserve(10);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.attributes.reserve(additional);
    }

    /// Adds attributes to the list from an attribute list definition, which is parsed to arguments.
    /// `source` should generally be from the `attributes` field of an [`AttributeListDefinition`].
    ///
    /// [`AttributeListDefinition`]: crate::node::AttributeListDefinition
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::node::{AttributeListDefinition, Tag};
    ///
    /// let mut attrs = mxp::AttributeList::new();
    /// let definition: AttributeListDefinition =
    ///     Tag::parse("!ATTLIST boldtext 'color=red flags'", true).unwrap().try_into().unwrap();
    ///
    /// attrs.append(definition.attributes).unwrap();
    /// assert_eq!(attrs.get("color"), Some("red"));
    /// ```
    pub fn append(&mut self, source: &str) -> crate::Result<()> {
        let len = self.len();
        let result = self.append_args(ArgumentParser::new(source));
        if result.is_err() {
            self.truncate(len);
        }
        result
    }

    /// Removes all attributes from the list with a positional index greater than or equal to the
    /// specified length.
    pub(crate) fn truncate(&mut self, i: usize) {
        self.attributes.retain(|_, value| value.position < i);
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
    fn append_args(&mut self, args: ArgumentParser) -> crate::Result<()> {
        self.reserve(args.size_hint().1.unwrap());
        for entry in args {
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

impl TryFrom<ArgumentParser<'_>> for AttributeList {
    type Error = Error;

    fn try_from(value: ArgumentParser) -> crate::Result<Self> {
        let mut list = AttributeList::new();
        list.append_args(value)?;
        Ok(list)
    }
}

impl FromStr for AttributeList {
    type Err = Error;

    fn from_str(s: &str) -> crate::Result<Self> {
        ArgumentParser::new(s).try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt() {
        let args = ArgumentParser::new("EL RName FLAG=\"Room Name\" Owner Locked=0");
        let formatted = AttributeList::try_from(args).unwrap().to_string();
        assert_eq!(formatted, "EL RName FLAG=\"Room Name\" Owner Locked=0");
    }
}
