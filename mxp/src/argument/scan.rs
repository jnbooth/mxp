use std::borrow::Cow;
use std::marker::PhantomData;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::{slice, str};

use flagset::{FlagSet, Flags};

use super::keyword_filter::{KeywordFilter, NoKeywords};
use crate::collections::CaseFoldMap;
use crate::parser::{Error, ErrorKind};

pub trait Decoder {
    fn decode<'a, F>(&self, s: &'a str) -> crate::Result<Cow<'a, str>>
    where
        F: KeywordFilter;
}

impl<D: Decoder> Decoder for &D {
    fn decode<'a, F>(&self, s: &'a str) -> crate::Result<Cow<'a, str>>
    where
        F: KeywordFilter,
    {
        D::decode::<F>(self, s)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Scan<'a, D, F = NoKeywords> {
    decoder: D,
    inner: slice::Iter<'a, Cow<'a, str>>,
    named: &'a CaseFoldMap<'a, Cow<'a, str>>,
    phantom: PhantomData<F>,
}

impl<'a, D, F> Scan<'a, D, F>
where
    D: Decoder,
    F: KeywordFilter,
{
    pub fn new(
        decoder: D,
        positional: &'a [Cow<'a, str>],
        named: &'a CaseFoldMap<'a, Cow<'a, str>>,
    ) -> Self {
        Self {
            decoder,
            inner: positional.iter(),
            named,
            phantom: PhantomData,
        }
    }

    fn with_filter<FNew>(self) -> Scan<'a, D, FNew> {
        Scan {
            decoder: self.decoder,
            inner: self.inner,
            named: self.named,
            phantom: PhantomData,
        }
    }

    pub fn with_keywords<E: Flags + FromStr>(self) -> KeywordScan<'a, D, E> {
        KeywordScan {
            inner: self.with_filter(),
            keywords: FlagSet::empty(),
        }
    }

    fn decode<S>(&self, s: Option<&'a S>) -> crate::Result<Option<Cow<'a, str>>>
    where
        S: AsRef<str> + ?Sized,
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
        self.decode(self.named.get(name))
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

pub(crate) struct KeywordScan<'a, D, K: Flags> {
    inner: Scan<'a, D, K>,
    keywords: FlagSet<K>,
}

impl<'a, D, K: Flags> Deref for KeywordScan<'a, D, K> {
    type Target = Scan<'a, D, K>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D, K: Flags> DerefMut for KeywordScan<'_, D, K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, D, K> KeywordScan<'a, D, K>
where
    D: Decoder,
    K: Flags + FromStr,
{
    fn next_non_keyword(&mut self) -> Option<&'a str> {
        for arg in &mut self.inner.inner {
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
        keywords.extend(self.inner.inner.filter_map(|arg| arg.parse::<K>().ok()));
        keywords
    }
}

pub(crate) trait ExpectArg {
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
