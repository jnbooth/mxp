use std::borrow::Cow;
use std::slice;

use uncased::Uncased;

use super::scan::{Decoder, Scan};
use super::validation::validate;
use super::words::Words;
use crate::collections::CaseFoldMap;
use crate::keyword::KeywordFilter;
use crate::parse::ArgumentMatcher;
use crate::{Error, ErrorKind};

/// Parsed arguments of an MXP command.
///
/// Arguments may be positional or named. For example, in the MXP command
/// `<SOUND "ouch.wav" 50 T="combat" 2 P=80>`, `"ouch.wav"`, `50`, and `2` are positional arguments,
/// and `T="combat"` and `P=80` are named arguments.
///
/// See [MXP specification: Attributes](https://www.zuggsoft.com/zmud/mxp.htm#ATTLIST).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arguments<'a, S = &'a str> {
    positional: Vec<S>,
    named: CaseFoldMap<'a, S>,
}

impl<S> Default for Arguments<'_, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Arguments<'_, S> {
    /// Constructs a new, empty `Arguments<S>`.
    pub fn new() -> Self {
        Self {
            positional: Vec::new(),
            named: CaseFoldMap::new(),
        }
    }

    /// Returns `true` if there are no parsed arguments.
    pub fn is_empty(&self) -> bool {
        self.positional.is_empty() && self.named.is_empty()
    }
}

impl<'a, S: AsRef<str>> Arguments<'a, S> {
    /// Finds the value of an entity, using an element's attribute list to identify arguments
    /// and provide default values.
    pub(crate) fn find_from_attributes<K: KeywordFilter>(
        &'a self,
        entity: &str,
        attributes: &'a Arguments<'static, String>,
    ) -> Option<&'a str> {
        if let Some(named) = attributes.named.get(entity) {
            return match self.named.get(entity) {
                Some(entity) => Some(entity.as_ref()),
                None => Some(named),
            };
        }
        let position =
            K::iter(&attributes.positional).position(|attr| attr.eq_ignore_ascii_case(entity))?;
        match K::iter(&self.positional).nth(position) {
            Some(attr) => Some(attr.as_ref()),
            None => Some(""),
        }
    }

    pub(crate) fn matcher(&self) -> ArgumentMatcher<'_, slice::Iter<'_, S>, S> {
        ArgumentMatcher::new(&self.positional, &self.named)
    }

    pub(crate) fn scan<D: Decoder>(&self, decoder: D) -> Scan<'_, D, S> {
        Scan::new(decoder, &self.positional, &self.named)
    }

    pub(crate) fn extend<'b, T>(&mut self, mut iter: Words<'b>) -> crate::Result<()>
    where
        T: From<&'b str> + Into<S> + Into<Uncased<'a>>,
    {
        while let Some(name) = iter.next() {
            if name == "/" {
                if iter.next().is_none() {
                    return Ok(());
                }
                return Err(Error::new(name, ErrorKind::InvalidArgumentName));
            }
            if iter.as_str().starts_with('=') {
                validate(name, ErrorKind::InvalidArgumentName)?;
                iter.next();
                let val = iter
                    .next()
                    .ok_or_else(|| Error::new(format!("{name}="), ErrorKind::EmptyArgument))?;
                self.named.insert(T::from(name), T::from(val).into());
            } else {
                self.positional.push(T::from(name).into());
            }
        }
        Ok(())
    }
}

impl<'a> Arguments<'a> {
    pub fn parse(source: &'a str) -> crate::Result<Self> {
        Words::new(source).try_into()
    }
}

impl<'a> TryFrom<Words<'a>> for Arguments<'a> {
    type Error = crate::Error;

    fn try_from(value: Words<'a>) -> crate::Result<Self> {
        let mut this = Self::new();
        this.extend::<&str>(value)?;
        Ok(this)
    }
}

impl<'a> TryFrom<Words<'a>> for Arguments<'a, Cow<'a, str>> {
    type Error = crate::Error;

    fn try_from(value: Words<'a>) -> crate::Result<Self> {
        let mut this = Self::new();
        this.extend::<&str>(value)?;
        Ok(this)
    }
}

impl TryFrom<Words<'_>> for Arguments<'static, String> {
    type Error = crate::Error;

    fn try_from(value: Words<'_>) -> crate::Result<Self> {
        let mut this = Self::new();
        this.extend::<String>(value)?;
        Ok(this)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments() {
        let words = Words::new(r#"EL RName '<FONT COLOR=Red><B>' FLAG="RoomName""#);
        let args: Arguments = words.try_into().unwrap();
        let expected = Arguments {
            positional: vec!["EL", "RName", "<FONT COLOR=Red><B>"],
            named: [("flag", "RoomName")].iter().copied().collect(),
        };
        assert_eq!(args, expected);
    }
}
