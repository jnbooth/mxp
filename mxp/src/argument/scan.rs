use super::keyword_filter::{KeywordFilter, NoKeywords};
use crate::parser::{Error, ErrorKind};
use casefold::ascii::CaseFoldMap;
use flagset::{FlagSet, Flags};
use std::borrow::Cow;
use std::marker::PhantomData;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::{slice, str};

pub trait Decoder {
    fn decode<'a, F: KeywordFilter>(&self, s: &'a str) -> crate::Result<Cow<'a, str>>;
}

impl<D: Decoder> Decoder for &D {
    fn decode<'a, F: KeywordFilter>(&self, s: &'a str) -> crate::Result<Cow<'a, str>> {
        D::decode::<F>(self, s)
    }
}

#[derive(Clone, Debug)]
pub struct Scan<'a, D, S, F = NoKeywords> {
    decoder: D,
    inner: slice::Iter<'a, S>,
    named: &'a CaseFoldMap<String, S>,
    __marker: PhantomData<F>,
}

impl<'a, D: Decoder, S: AsRef<str>, F: KeywordFilter> Scan<'a, D, S, F> {
    pub(crate) fn new(decoder: D, positional: &'a [S], named: &'a CaseFoldMap<String, S>) -> Self {
        Self {
            decoder,
            inner: positional.iter(),
            named,
            __marker: PhantomData,
        }
    }

    pub fn with_filter<FNew>(self) -> Scan<'a, D, S, FNew> {
        Scan {
            decoder: self.decoder,
            inner: self.inner,
            named: self.named,
            __marker: PhantomData,
        }
    }

    pub fn with_keywords<E: Flags + FromStr>(self) -> KeywordScan<'a, D, S, E> {
        KeywordScan {
            inner: self.with_filter(),
            keywords: FlagSet::default(),
        }
    }

    fn decode<SD>(&self, s: Option<&'a SD>) -> crate::Result<Option<Cow<'a, str>>>
    where
        SD: AsRef<str> + ?Sized,
    {
        match s {
            Some(s) => self.decoder.decode::<F>(s.as_ref()).map(Option::Some),
            None => Ok(None),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    fn get(&self, name: &str) -> crate::Result<Option<Cow<'a, str>>> {
        self.decode(self.named.get(name).map(AsRef::as_ref))
    }

    pub fn next(&mut self) -> crate::Result<Option<Cow<'a, str>>> {
        let next = self.inner.next();
        self.decode(next)
    }

    pub fn next_or(&mut self, name: &str) -> crate::Result<Option<Cow<'a, str>>> {
        match self.get(name)? {
            Some(value) => Ok(Some(value)),
            None => self.next(),
        }
    }
}

pub struct KeywordScan<'a, D, S, K: Flags> {
    inner: Scan<'a, D, S, K>,
    keywords: FlagSet<K>,
}

impl<'a, D, S: AsRef<str>, K: Flags> Deref for KeywordScan<'a, D, S, K> {
    type Target = Scan<'a, D, S, K>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, D, S: AsRef<str>, K: Flags> DerefMut for KeywordScan<'a, D, S, K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, D: Decoder, S: AsRef<str>, K: Flags + FromStr> KeywordScan<'a, D, S, K> {
    pub fn keywords(&self) -> FlagSet<K> {
        self.keywords
    }

    fn next_non_keyword(&mut self) -> Option<&'a str> {
        for arg in &mut self.inner.inner {
            let arg = arg.as_ref();
            if let Ok(keyword) = arg.parse::<K>() {
                self.keywords |= keyword;
            } else {
                return Some(arg);
            }
        }
        None
    }

    pub fn next(&mut self) -> crate::Result<Option<Cow<'a, str>>> {
        let next = self.next_non_keyword();
        self.decode(next)
    }

    pub fn next_or(&mut self, name: &str) -> crate::Result<Option<Cow<'a, str>>> {
        match self.get(name)? {
            Some(value) => Ok(Some(value)),
            None => self.next(),
        }
    }

    pub fn into_keywords(self) -> FlagSet<K> {
        let mut keywords = self.keywords;
        keywords.extend(
            self.inner
                .inner
                .filter_map(|arg| arg.as_ref().parse::<K>().ok()),
        );
        keywords
    }
}

pub trait ExpectArg {
    type Arg;

    fn expect_some(self, name: &str) -> crate::Result<Self::Arg>;
    fn expect_number<T>(self) -> crate::Result<Option<T>>
    where
        Self::Arg: AsRef<str>,
        T: FromStr<Err = ParseIntError>;
}

impl<S> ExpectArg for Option<S> {
    type Arg = S;

    fn expect_some(self, name: &str) -> crate::Result<Self::Arg> {
        match self {
            Some(arg) => Ok(arg),
            None => Err(Error::new(name, ErrorKind::IncompleteArguments)),
        }
    }

    fn expect_number<T>(self) -> crate::Result<Option<T>>
    where
        Self::Arg: AsRef<str>,
        T: FromStr<Err = ParseIntError>,
    {
        let Some(arg) = self else {
            return Ok(None);
        };
        match arg.as_ref().parse() {
            Ok(parsed) => Ok(Some(parsed)),
            Err(_) => Err(Error::new(arg.as_ref(), ErrorKind::InvalidNumber)),
        }
    }
}
