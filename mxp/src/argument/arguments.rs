use casefold::ascii::CaseFoldMap;
use enumeration::EnumSet;

use super::keyword::Keyword;
use super::scan::{Decoder, Scan};
use crate::parser::{validate, Error as MxpError, ParseError, Words};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Arguments {
    positional: Vec<String>,
    named: CaseFoldMap<String, String>,
    keywords: EnumSet<Keyword>,
}

impl Arguments {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn predefined(
        positional: Vec<String>,
        named: CaseFoldMap<String, String>,
        keywords: EnumSet<Keyword>,
    ) -> Self {
        Self {
            positional,
            named,
            keywords,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.positional.is_empty() && self.named.is_empty()
    }

    pub fn has_keyword(&self, k: Keyword) -> bool {
        self.keywords.contains(k)
    }

    pub fn find_attribute<'a>(&'a self, entity: &str, other: &'a Self) -> Option<&'a str> {
        if let Some(named) = self.named.get(entity) {
            return Some(other.named.get(entity).unwrap_or(named).as_str());
        }
        let position = self
            .positional
            .iter()
            .position(|attr| attr.eq_ignore_ascii_case(entity))?;
        match other.positional.get(position) {
            Some(attr) => Some(attr.as_str()),
            None => Some(""),
        }
    }

    pub fn scan<D: Decoder>(&self, decoder: D) -> Scan<D> {
        Scan {
            decoder,
            inner: self.positional.iter().map(String::as_str),
            keywords: self.keywords,
            named: &self.named,
        }
    }

    pub fn parse(iter: Words) -> Result<Self, ParseError> {
        let mut this = Self::new();
        this.append(iter)?;
        Ok(this)
    }

    pub(crate) fn append(&mut self, mut iter: Words) -> Result<(), ParseError> {
        while let Some(name) = iter.next() {
            if name == "/" {
                if iter.next().is_none() {
                    return Ok(());
                } else {
                    return Err(ParseError::new(name, MxpError::InvalidArgumentName));
                }
            }
            if iter.as_str().starts_with('=') {
                validate(name, MxpError::InvalidArgumentName)?;
                iter.next();
                let val = iter
                    .next()
                    .ok_or_else(|| ParseError::new(name, MxpError::NoArgument))?;
                self.named.insert(name.to_lowercase(), val.to_owned());
            } else if let Some(keyword) = Keyword::parse(name) {
                self.keywords.insert(keyword);
            } else {
                self.positional.push(name.to_owned());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use casefold::ascii::CaseFold;

    use super::*;

    #[test]
    fn arguments() {
        let words = Words::new(r#"EL RName '<FONT COLOR=Red><B>' FLAG="RoomName""#);
        let args = Arguments::parse(words).unwrap();
        let should_be = Arguments {
            positional: ["EL", "RName", "<FONT COLOR=Red><B>"]
                .iter()
                .map(ToString::to_string)
                .collect(),
            named: [("flag", "RoomName")]
                .iter()
                .map(|(k, v)| (CaseFold::new(k.to_string()), v.to_string()))
                .collect(),
            keywords: Default::default(),
        };
        assert_eq!(args, should_be);
    }
}
