use std::borrow::Cow;
use std::marker::PhantomData;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::{slice, str};

use flagset::{FlagSet, Flags};

use super::error::{Error, ErrorKind};
use crate::collections::CaseFoldMap;
use crate::color::RgbColor;
use crate::entity::{DecodedEntity, Entity};
use crate::keyword::KeywordFilter;

pub trait Decoder {
    fn get_entity<F>(&self, name: &str) -> Option<&str>
    where
        F: KeywordFilter;

    fn decode_entity<F>(&self, entity: &str) -> crate::Result<Option<DecodedEntity<'_>>>
    where
        F: KeywordFilter,
    {
        let (start, radix) = match entity.as_bytes() {
            [b'#', b'x', ..] => (2, 16),
            [b'#', ..] => (1, 10),
            _ => return Ok(self.get_entity::<F>(entity).map(Into::into)),
        };
        let Ok(code) = u32::from_str_radix(&entity[start..], radix) else {
            return Err(Error::new(entity, ErrorKind::InvalidEntityNumber));
        };
        match char::from_u32(code) {
            Some('\0'..='\x08' | '\x0a'..='\x1f' | '\x7f'..='\u{9f}') | None => {
                Err(Error::new(entity, ErrorKind::DisallowedEntityNumber))
            }
            Some(c) => Ok(Some(c.into())),
        }
    }
}

impl<D: Decoder> Decoder for &D {
    fn get_entity<F: KeywordFilter>(&self, name: &str) -> Option<&str> {
        D::get_entity::<F>(self, name)
    }

    fn decode_entity<F>(&self, entity: &str) -> crate::Result<Option<DecodedEntity<'_>>>
    where
        F: KeywordFilter,
    {
        D::decode_entity::<F>(self, entity)
    }
}

impl Decoder for () {
    fn get_entity<F: KeywordFilter>(&self, name: &str) -> Option<&str> {
        Entity::global(name)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Scan<'a, D, F = ()> {
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
        let Some(s) = s else {
            return Ok(None);
        };
        let mut s = s.as_ref();
        let mut res = String::new();
        while let Some(start) = s.find('&') {
            if start > 0 {
                res.push_str(&s[..start]);
            }
            s = &s[start..];
            let end = s
                .find(';')
                .ok_or_else(|| Error::new(s, ErrorKind::NoClosingSemicolon))?;
            match self.decoder.decode_entity::<F>(&s[1..end])? {
                Some(decoded) => decoded.push_to(&mut res),
                None => res.push_str(&s[..=end]),
            }
            s = &s[end + 1..];
        }
        if res.is_empty() {
            return Ok(Some(Cow::Borrowed(s)));
        }
        if !s.is_empty() {
            res.push_str(s);
        }
        Ok(Some(Cow::Owned(res)))
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
    K: KeywordFilter + Flags + FromStr,
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

    fn color(self) -> Option<RgbColor>
    where
        Self::Arg: AsRef<str>;
    fn expect_some(self, name: &str) -> crate::Result<Self::Arg>;
    fn expect_number<T>(self) -> crate::Result<Option<T>>
    where
        Self::Arg: AsRef<str>,
        T: FromStr<Err = ParseIntError>;
}

impl<S> ExpectArg for Option<S> {
    type Arg = S;

    fn color(self) -> Option<RgbColor>
    where
        Self::Arg: AsRef<str>,
    {
        RgbColor::named(self?.as_ref())
    }

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
