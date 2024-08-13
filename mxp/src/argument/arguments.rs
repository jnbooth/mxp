use std::iter::Filter;
use std::slice;
use std::str::FromStr;

use casefold::ascii::CaseFoldMap;

use super::scan::{Decoder, Scan};
use crate::parser::{validate, Error, ErrorKind, Words};

pub trait KeywordFilter {
    type Iter<'a>: Iterator<Item = &'a String>;

    fn iter(args: &[String]) -> Self::Iter<'_>;
}

pub struct NoKeywords;

impl KeywordFilter for NoKeywords {
    type Iter<'a> = slice::Iter<'a, String>;

    fn iter(args: &[String]) -> Self::Iter<'_> {
        args.iter()
    }
}

impl<K: FromStr> KeywordFilter for K {
    type Iter<'a> = Filter<slice::Iter<'a, String>, fn(&&String) -> bool>;

    fn iter(args: &[String]) -> Self::Iter<'_> {
        args.iter().filter(|arg| K::from_str(arg).is_err())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Arguments {
    positional: Vec<String>,
    named: CaseFoldMap<String, String>,
}

impl Arguments {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.positional.is_empty() && self.named.is_empty()
    }

    pub fn find_attribute<'a, F: KeywordFilter>(
        &'a self,
        entity: &str,
        other: &'a Self,
    ) -> Option<&'a str> {
        if let Some(named) = self.named.get(entity) {
            return Some(other.named.get(entity).unwrap_or(named).as_str());
        }
        let position =
            F::iter(&self.positional).position(|attr| attr.eq_ignore_ascii_case(entity))?;
        match F::iter(&other.positional).nth(position) {
            Some(attr) => Some(attr.as_str()),
            None => Some(""),
        }
    }

    pub fn scan<D: Decoder>(&self, decoder: D) -> Scan<D> {
        Scan::new(decoder, &self.positional, &self.named)
    }

    pub fn parse(iter: Words) -> crate::Result<Self> {
        let mut this = Self::new();
        this.append(iter)?;
        Ok(this)
    }

    pub(crate) fn append(&mut self, mut iter: Words) -> crate::Result<()> {
        while let Some(name) = iter.next() {
            if name == "/" {
                if iter.next().is_none() {
                    return Ok(());
                } else {
                    return Err(Error::new(name, ErrorKind::InvalidArgumentName));
                }
            }
            if iter.as_str().starts_with('=') {
                validate(name, ErrorKind::InvalidArgumentName)?;
                iter.next();
                let val = iter
                    .next()
                    .ok_or_else(|| Error::new(name, ErrorKind::NoArgument))?;
                self.named.insert(name.to_lowercase(), val.to_owned());
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
        };
        assert_eq!(args, should_be);
    }
}
