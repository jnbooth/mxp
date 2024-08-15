use super::arguments::{KeywordFilter, NoKeywords};
use crate::color::RgbColor;
use crate::entity::Atom;
use crate::keyword::{EntityKeyword, MxpKeyword};
use crate::parser::{Error, ErrorKind};
use casefold::ascii::CaseFoldMap;
use enumeration::{Enum, EnumSet};
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::{slice, str};

pub trait Decoder {
    type Output<'a>: AsRef<str>;

    fn decode<'a, F: KeywordFilter>(&self, s: &'a str) -> crate::Result<Self::Output<'a>>;
}

impl<D: Decoder> Decoder for &D {
    type Output<'a> = D::Output<'a>;

    fn decode<'a, F: KeywordFilter>(&self, s: &'a str) -> crate::Result<Self::Output<'a>> {
        D::decode::<F>(self, s)
    }
}

#[derive(Clone, Debug)]
pub struct Scan<'a, D, F = NoKeywords> {
    decoder: D,
    inner: slice::Iter<'a, String>,
    named: &'a CaseFoldMap<String, String>,
    __marker: PhantomData<F>,
}

impl<'a, D: Decoder, F: KeywordFilter> Scan<'a, D, F> {
    pub(crate) fn new(
        decoder: D,
        positional: &'a [String],
        named: &'a CaseFoldMap<String, String>,
    ) -> Self {
        Self {
            decoder,
            inner: positional.iter(),
            named,
            __marker: PhantomData,
        }
    }

    pub fn with_filter<FNew>(self) -> Scan<'a, D, FNew> {
        Scan {
            decoder: self.decoder,
            inner: self.inner,
            named: self.named,
            __marker: PhantomData,
        }
    }

    pub fn with_keywords<E: Enum + FromStr>(self) -> KeywordScan<'a, D, E> {
        KeywordScan {
            inner: self.with_filter(),
            keywords: EnumSet::new(),
        }
    }

    fn decode<S>(&self, s: Option<&'a S>) -> crate::Result<Option<D::Output<'a>>>
    where
        S: Borrow<str> + ?Sized,
    {
        match s {
            Some(s) => self.decoder.decode::<F>(s.borrow()).map(Option::Some),
            None => Ok(None),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    fn get(&self, name: &str) -> crate::Result<Option<D::Output<'a>>> {
        self.decode(self.named.get(name))
    }

    pub fn next_number_or<T>(&mut self, name: &str) -> crate::Result<Option<T>>
    where
        T: FromStr<Err = ParseIntError>,
    {
        let output = match self.next_or(name) {
            Ok(Some(output)) => output,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        };
        let output = output.as_ref();
        match output.parse() {
            Ok(parsed) => Ok(Some(parsed)),
            Err(_) => Err(Error::new(output, ErrorKind::InvalidNumber)),
        }
    }

    pub fn next(&mut self) -> crate::Result<Option<D::Output<'a>>> {
        let next = self.inner.next();
        self.decode(next)
    }

    pub fn next_or(&mut self, name: &str) -> crate::Result<Option<D::Output<'a>>> {
        match self.get(name)? {
            Some(value) => Ok(Some(value)),
            None => self.next(),
        }
    }
}

pub struct KeywordScan<'a, D, K: Enum> {
    inner: Scan<'a, D, K>,
    keywords: EnumSet<K>,
}

impl<'a, D, K: Enum> Deref for KeywordScan<'a, D, K> {
    type Target = Scan<'a, D, K>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, D, K: Enum> DerefMut for KeywordScan<'a, D, K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, D: Decoder, K: Enum + FromStr> KeywordScan<'a, D, K> {
    pub fn keywords(&self) -> EnumSet<K> {
        self.keywords
    }

    fn next_non_keyword(&mut self) -> Option<&'a str> {
        for arg in &mut self.inner.inner {
            if let Ok(keyword) = arg.parse() {
                self.keywords.insert(keyword);
            } else {
                return Some(arg);
            }
        }
        None
    }

    pub fn next(&mut self) -> crate::Result<Option<D::Output<'a>>> {
        let next = self.next_non_keyword();
        self.decode(next)
    }

    pub fn next_or(&mut self, name: &str) -> crate::Result<Option<D::Output<'a>>> {
        match self.get(name)? {
            Some(value) => Ok(Some(value)),
            None => self.next(),
        }
    }

    pub fn into_keywords(self) -> EnumSet<K> {
        let mut keywords = self.keywords;
        for keyword in self.inner.inner.filter_map(|arg| arg.parse().ok()) {
            keywords.insert(keyword);
        }
        keywords
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AfkArgs<S> {
    pub challenge: Option<S>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for AfkArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            challenge: scanner.next_or("challenge")?,
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorArgs {
    pub fore: Option<RgbColor>,
    pub back: Option<RgbColor>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for ColorArgs {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            fore: scanner
                .next_or("fore")?
                .and_then(|fore| RgbColor::named(fore.as_ref())),
            back: scanner
                .next_or("back")?
                .and_then(|back| RgbColor::named(back.as_ref())),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MxpArgs {
    pub keywords: EnumSet<MxpKeyword>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for MxpArgs {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let scanner = scanner.with_keywords();
        Ok(Self {
            keywords: scanner.into_keywords(),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SupportArgs {
    pub supported: Vec<u8>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for SupportArgs {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> Result<Self, Self::Error> {
        let mut questions = Vec::with_capacity(scanner.len());
        while let Some(question) = scanner.next()? {
            questions.push(question);
        }
        let mut supported = Vec::new();
        Atom::fmt_supported(&mut supported, &questions);
        Ok(Self { supported })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarArgs<S> {
    pub variable: S,
    pub keywords: EnumSet<EntityKeyword>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for VarArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        Ok(Self {
            variable: scanner
                .next()?
                .ok_or_else(|| Error::new("variable", ErrorKind::IncompleteArguments))?,
            keywords: scanner.into_keywords(),
        })
    }
}
