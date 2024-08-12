use std::collections::hash_map;
use std::iter::{Chain, Enumerate, Map};
use std::{slice, str};

use casefold::ascii::{CaseFold, CaseFoldMap};
use enumeration::{Enum, EnumSet};

use super::error::{Error as MxpError, ParseError};
use super::scan::{Decoder, Scan};
use super::validation::validate;
use super::words::Words;

pub type Argument = String;
pub type Arg = str;

/// MXP elements can have both positional and named arguments.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArgumentIndex<'a> {
    Positional(usize),
    Named(&'a str),
}
impl<'a> From<usize> for ArgumentIndex<'a> {
    fn from(value: usize) -> Self {
        Self::Positional(value)
    }
}
impl<'a> From<&'a str> for ArgumentIndex<'a> {
    fn from(value: &'a str) -> Self {
        Self::Named(value)
    }
}
impl<'a> ArgumentIndex<'a> {
    pub fn is_positional(self) -> bool {
        match self {
            Self::Positional(_) => true,
            Self::Named(_) => false,
        }
    }

    pub fn is_named(self) -> bool {
        match self {
            Self::Positional(_) => false,
            Self::Named(_) => true,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Keyword {
    Delete,
    Open,
    Empty,
    Prompt,
    Off,
    DefaultOpen,
    DefaultSecure,
    DefaultLocked,
    UseNewlines,
    IgnoreNewlines,
    IsMap,
}

impl Keyword {
    fn parse(s: &str) -> Option<Self> {
        match_ci! {s,
            "delete" => Some(Self::Delete),
            "open" => Some(Self::Open),
            "empty" => Some(Self::Empty),
            "prompt" => Some(Self::Prompt),
            "off" => Some(Self::Off),
            "defaultopen" => Some(Self::DefaultOpen),
            "defaultsecure" => Some(Self::DefaultSecure),
            "defaultlocked" => Some(Self::DefaultLocked),
            "usenewlines" => Some(Self::UseNewlines),
            "ignorenewlines" => Some(Self::IgnoreNewlines),
            "ismap" => Some(Self::IsMap),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Arguments {
    positional: Vec<Argument>,
    named: CaseFoldMap<String, Argument>,
    keywords: EnumSet<Keyword>,
}

impl Arguments {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.positional.len() + self.named.len()
    }

    pub fn is_empty(&self) -> bool {
        self.positional.is_empty() && self.named.is_empty()
    }

    pub fn clear(&mut self) {
        self.positional.clear();
        self.named.clear();
        self.keywords.clear();
    }

    pub fn get<'a, Idx: Into<ArgumentIndex<'a>>>(&self, idx: Idx) -> Option<&Arg> {
        match idx.into() {
            ArgumentIndex::Positional(i) => self.positional.get(i),
            ArgumentIndex::Named(name) => self.named.get(name),
        }
        .map(String::as_str)
    }

    pub fn get_mut<'a, Idx: Into<ArgumentIndex<'a>>>(&mut self, idx: Idx) -> Option<&mut Argument> {
        match idx.into() {
            ArgumentIndex::Positional(i) => self.positional.get_mut(i),
            ArgumentIndex::Named(name) => self.named.get_mut(name),
        }
    }

    pub fn has_keyword(&self, k: Keyword) -> bool {
        self.keywords.contains(k)
    }

    pub fn push(&mut self, arg: Argument) {
        self.positional.push(arg);
    }

    pub fn set(&mut self, key: &str, arg: Argument) {
        self.named.insert(key.to_owned(), arg);
    }

    pub fn iter(&self) -> impl Iterator<Item = (ArgumentIndex, &Arg)> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (ArgumentIndex, &mut Argument)> {
        self.into_iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &Arg> {
        self.positional
            .iter()
            .chain(self.named.values())
            .map(String::as_str)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Argument> {
        self.positional.iter_mut().chain(self.named.values_mut())
    }

    pub fn scan<D: Decoder>(&self, decoder: D) -> Scan<D> {
        Scan {
            decoder,
            inner: self.positional.iter().map(String::as_str),
            keywords: self.keywords,
            named: &self.named,
        }
    }

    pub fn parse(tag: &str) -> Result<Self, ParseError> {
        Self::parse_words(Words::new(tag))
    }

    pub fn parse_words(iter: Words) -> Result<Self, ParseError> {
        let mut this = Self::new();
        this.append(iter)?;
        Ok(this)
    }

    pub fn append(&mut self, mut iter: Words) -> Result<(), ParseError> {
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

// Just some nicknames for internal use
type Index<'a> = ArgumentIndex<'a>;

type IterItem<'a> = (Index<'a>, &'a Arg);
type IterItemMut<'a> = (Index<'a>, &'a mut Argument);

type PositionalEntry<'a> = (usize, &'a Argument);
type PositionalEntryMut<'a> = (usize, &'a mut Argument);

type NamedEntry<'a> = (&'a CaseFold<String>, &'a Argument);
type NamedEntryMut<'a> = (&'a CaseFold<String>, &'a mut Argument);

type Iter<'a, A, B, SliceIter, MapIter> = Chain<
    Map<Enumerate<SliceIter>, fn((usize, A)) -> (Index<'a>, B)>,
    Map<MapIter, fn((&'a CaseFold<String>, A)) -> (Index<'a>, B)>,
>;
type IntoIter<'a> = Iter<
    'a,
    &'a Argument,
    &'a Arg,
    slice::Iter<'a, Argument>,
    hash_map::Iter<'a, CaseFold<String>, Argument>,
>;
type IntoIterMut<'a> = Iter<
    'a,
    &'a mut Argument,
    &'a mut Argument,
    slice::IterMut<'a, Argument>,
    hash_map::IterMut<'a, CaseFold<String>, Argument>,
>;

impl<'a> IntoIterator for &'a Arguments {
    type IntoIter = IntoIter<'a>;
    type Item = IterItem<'a>;

    fn into_iter(self) -> IntoIter<'a> {
        let positional = self
            .positional
            .iter()
            .enumerate()
            .map((|(i, x)| (Index::Positional(i), x)) as fn(PositionalEntry) -> IterItem);

        let named = self
            .named
            .iter()
            .map((|(k, v)| (Index::Named(k.as_str()), v)) as fn(NamedEntry) -> IterItem);

        positional.chain(named)
    }
}

impl<'a> IntoIterator for &'a mut Arguments {
    type IntoIter = IntoIterMut<'a>;
    type Item = IterItemMut<'a>;

    fn into_iter(self) -> IntoIterMut<'a> {
        let positional = self
            .positional
            .iter_mut()
            .enumerate()
            .map((|(i, x)| (Index::Positional(i), x)) as fn(PositionalEntryMut) -> IterItemMut);

        let named = self
            .named
            .iter_mut()
            .map((|(k, v)| (Index::Named(k.as_str()), v)) as fn(NamedEntryMut) -> IterItemMut);

        positional.chain(named)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments() {
        let args = Arguments::parse(r#"EL RName '<FONT COLOR=Red><B>' FLAG="RoomName""#).unwrap();
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
