use std::fmt;
use std::str::FromStr;

use crate::arguments::Arguments;
use crate::parse::Words;
use crate::{CaseFoldMap, Error, ErrorKind};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Attribute {
    position: usize,
    value: Option<String>,
}

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

    /// Returns true if the list contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut attrs = mxp::AttributeList::new();
    /// assert!(attrs.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        Some(self.attributes.get(name)?.value.as_ref()?.as_str())
    }

    pub fn position(&self, name: &str) -> Option<usize> {
        Some(self.attributes.get(name)?.position)
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

    pub(crate) fn append(&mut self, iter: Words) -> crate::Result<()> {
        let size = count_args(iter.clone());
        self.attributes.reserve(size);
        let mut in_named = false;
        let mut last_name = "";
        for word in iter {
            if in_named {
                let Some(attribute) = self.attributes.get_mut(last_name) else {
                    return Err(Error::new(
                        format!("={word}"),
                        ErrorKind::MissingArgumentName,
                    ));
                };
                attribute.value = Some(word.to_owned());
                continue;
            }
            if word == "=" {
                in_named = true;
                continue;
            }
            last_name = word;
            self.attributes.insert(
                word.to_owned(),
                Attribute {
                    position: self.attributes.len(),
                    value: None,
                },
            );
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
