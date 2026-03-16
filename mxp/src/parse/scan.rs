use std::borrow::Cow;
use std::str::FromStr;
use std::{slice, str, vec};

use flagset::{FlagSet, Flags};

use super::validation::is_valid;
use crate::CaseFoldMap;
use crate::arguments::{ArgumentMatcher, OwnedArgumentMatcher};
use crate::entity::{DecodedEntity, Entity};
use crate::keyword::{KeywordFilter, KeywordFilterIter};
use crate::{Error, ErrorKind};

pub trait Decoder {
    fn get_entity<K: KeywordFilter>(&self, name: &str) -> Option<&str>;

    fn decode_entity<K: KeywordFilter>(&self, name: &str) -> crate::Result<DecodedEntity<'_>> {
        let (start, radix) = match name.as_bytes() {
            [b'#', b'x', ..] => (2, 16),
            [b'#', ..] => (1, 10),
            _ => {
                return match self.get_entity::<K>(name) {
                    Some(entity) => Ok(entity.into()),
                    None if is_valid(name) => Err(Error::new(name, ErrorKind::UnknownEntity)),
                    None => Err(Error::new(name, ErrorKind::InvalidEntityName)),
                };
            }
        };
        let Ok(code) = u32::from_str_radix(&name[start..], radix) else {
            return Err(Error::new(name, ErrorKind::InvalidEntityNumber));
        };
        match char::from_u32(code) {
            Some('\0'..='\x08' | '\x0a'..='\x1f' | '\x7f'..='\u{9f}') | None => {
                Err(Error::new(name, ErrorKind::IllegalEntityNumber))
            }
            Some(c) => Ok(c.into()),
        }
    }

    fn decode_string<'a, K: KeywordFilter>(&self, mut s: &'a str) -> crate::Result<Cow<'a, str>> {
        let mut res = String::new();
        while let Some((before, rest)) = s.split_once('&') {
            if !before.is_empty() {
                res.push_str(before);
            }
            let Some((entity, after)) = rest.split_once(';') else {
                return Err(Error::new(
                    format!("&{rest}"),
                    ErrorKind::NoClosingSemicolon,
                ));
            };
            self.decode_entity::<K>(entity)?.push_to(&mut res);
            s = after;
        }
        if res.is_empty() {
            return Ok(Cow::Borrowed(s));
        }
        if !s.is_empty() {
            res.push_str(s);
        }
        Ok(Cow::Owned(res))
    }
}

impl<D: Decoder> Decoder for &D {
    fn get_entity<K: KeywordFilter>(&self, name: &str) -> Option<&str> {
        D::get_entity::<K>(self, name)
    }

    fn decode_entity<K: KeywordFilter>(&self, entity: &str) -> crate::Result<DecodedEntity<'_>> {
        D::decode_entity::<K>(self, entity)
    }
}

/// Fallback `Decoder` that only looks up global entities (with [`Entity::global`]).
impl Decoder for () {
    fn get_entity<K: KeywordFilter>(&self, name: &str) -> Option<&str> {
        Entity::global(name)
    }
}

trait DecoderExt {
    fn decode_some<'a, K, S>(&self, s: Option<&'a S>) -> crate::Result<Option<Cow<'a, str>>>
    where
        K: KeywordFilter,
        S: ?Sized + AsRef<str>;
}

impl<D: Decoder> DecoderExt for D {
    fn decode_some<'a, K, S>(&self, s: Option<&'a S>) -> crate::Result<Option<Cow<'a, str>>>
    where
        K: KeywordFilter,
        S: ?Sized + AsRef<str>,
    {
        match s {
            Some(s) => Ok(Some(self.decode_string::<K>(s.as_ref())?)),
            None => Ok(None),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Scan<'a, D: Decoder, S: AsRef<str> = Cow<'a, str>> {
    decoder: D,
    inner: ArgumentMatcher<'a, slice::Iter<'a, S>, S>,
}

impl<'a, D: Decoder, S: AsRef<str>> Scan<'a, D, S> {
    pub fn new(decoder: D, positional: &'a [S], named: &'a CaseFoldMap<'a, S>) -> Self {
        Self {
            decoder,
            inner: ArgumentMatcher::new(positional, named),
        }
    }

    pub fn with_keywords<K: Flags + FromStr + KeywordFilter>(self) -> KeywordScan<'a, D, K, S> {
        KeywordScan {
            decoder: self.decoder,
            inner: self.inner.with_keywords(),
        }
    }

    pub fn next(&mut self) -> crate::Result<Option<Cow<'a, str>>> {
        self.decoder.decode_some::<(), _>(self.inner.next())
    }

    pub fn next_or(&mut self, name: &str) -> crate::Result<Option<Cow<'a, str>>> {
        self.decoder.decode_some::<(), _>(self.inner.next_or(name))
    }

    pub fn expect_end(mut self) -> crate::Result<()> {
        if let Some(next) = self.inner.next() {
            return Err(Error::new(next.as_ref(), ErrorKind::UnexpectedArgument));
        }
        Ok(())
    }
}

impl<'a, D: Decoder, S: AsRef<str>> Iterator for Scan<'a, D, S> {
    type Item = crate::Result<Cow<'a, str>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;
        Some(self.decoder.decode_string::<()>(next.as_ref()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[derive(Clone)]
pub(crate) struct OwnedScan<'a, D: Decoder> {
    decoder: D,
    inner: OwnedArgumentMatcher<'a, vec::IntoIter<&'a str>, &'a str>,
}

impl<'a, D: Decoder> OwnedScan<'a, D> {
    pub fn new(decoder: D, positional: Vec<&'a str>, named: CaseFoldMap<'a, &'a str>) -> Self {
        Self {
            decoder,
            inner: OwnedArgumentMatcher::new(positional, named),
        }
    }

    pub fn with_keywords<K: Flags + FromStr + KeywordFilter>(self) -> OwnedKeywordScan<'a, D, K> {
        OwnedKeywordScan {
            decoder: self.decoder,
            inner: self.inner.with_keywords(),
        }
    }

    pub fn next(&mut self) -> crate::Result<Option<Cow<'a, str>>> {
        self.decoder.decode_some::<(), _>(self.inner.next())
    }

    pub fn next_or(&mut self, name: &str) -> crate::Result<Option<Cow<'a, str>>> {
        self.decoder.decode_some::<(), _>(self.inner.next_or(name))
    }

    pub fn expect_end(mut self) -> crate::Result<()> {
        if let Some(next) = self.inner.next() {
            return Err(Error::new(next, ErrorKind::UnexpectedArgument));
        }
        Ok(())
    }
}

impl<'a, D: Decoder> Iterator for OwnedScan<'a, D> {
    type Item = crate::Result<Cow<'a, str>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;
        Some(self.decoder.decode_string::<()>(next.as_ref()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[derive(Clone)]
pub(crate) struct KeywordScan<'a, D, K, S = Cow<'a, str>>
where
    D: Decoder,
    K: KeywordFilter + Flags + FromStr,
    S: AsRef<str>,
{
    decoder: D,
    inner: ArgumentMatcher<'a, KeywordFilterIter<K, slice::Iter<'a, S>>, S>,
}

impl<'a, D, K, S> KeywordScan<'a, D, K, S>
where
    D: Decoder,
    K: KeywordFilter + Flags + FromStr,
    S: AsRef<str>,
{
    pub fn next(&mut self) -> crate::Result<Option<Cow<'a, str>>> {
        self.decoder.decode_some::<K, _>(self.inner.next())
    }

    pub fn next_or(&mut self, name: &str) -> crate::Result<Option<Cow<'a, str>>> {
        self.decoder.decode_some::<K, _>(self.inner.next_or(name))
    }

    pub fn into_keywords(self) -> Result<FlagSet<K>, K::Err> {
        self.inner.into_keywords()
    }
}

impl<'a, D, K, S> Iterator for KeywordScan<'a, D, K, S>
where
    D: Decoder,
    K: KeywordFilter + Flags + FromStr,
    S: AsRef<str>,
{
    type Item = crate::Result<Cow<'a, str>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;
        Some(self.decoder.decode_string::<K>(next.as_ref()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[derive(Clone)]
pub(crate) struct OwnedKeywordScan<'a, D, K>
where
    D: Decoder,
    K: KeywordFilter + Flags + FromStr,
{
    decoder: D,
    inner: OwnedArgumentMatcher<'a, KeywordFilterIter<K, vec::IntoIter<&'a str>>, &'a str>,
}

impl<'a, D, K> OwnedKeywordScan<'a, D, K>
where
    D: Decoder,
    K: KeywordFilter + Flags + FromStr,
{
    pub fn next(&mut self) -> crate::Result<Option<Cow<'a, str>>> {
        self.decoder.decode_some::<K, _>(self.inner.next())
    }

    pub fn next_or(&mut self, name: &str) -> crate::Result<Option<Cow<'a, str>>> {
        self.decoder.decode_some::<K, _>(self.inner.next_or(name))
    }

    pub fn into_keywords(self) -> Result<FlagSet<K>, K::Err> {
        self.inner.into_keywords()
    }
}

impl<'a, D, K> Iterator for OwnedKeywordScan<'a, D, K>
where
    D: Decoder,
    K: KeywordFilter + Flags + FromStr,
{
    type Item = crate::Result<Cow<'a, str>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;
        Some(self.decoder.decode_string::<K>(next.as_ref()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
