use std::borrow::Cow;
use std::str::FromStr;
use std::{slice, str};

use flagset::{FlagSet, Flags};

use super::error::{Error, ErrorKind};
use crate::collections::CaseFoldMap;
use crate::entity::{DecodedEntity, Entity};
use crate::keyword::{KeywordFilter, KeywordFilterIter};
use crate::parse::ArgumentMatcher;

pub trait Decoder {
    fn get_entity<K>(&self, name: &str) -> Option<&str>
    where
        K: KeywordFilter;

    fn decode_entity<K>(&self, entity: &str) -> crate::Result<DecodedEntity<'_>>
    where
        K: KeywordFilter,
    {
        let (start, radix) = match entity.as_bytes() {
            [b'#', b'x', ..] => (2, 16),
            [b'#', ..] => (1, 10),
            _ => {
                return match self.get_entity::<K>(entity) {
                    Some(entity) => Ok(entity.into()),
                    None => Err(Error::new(entity, ErrorKind::UnknownEntity)),
                };
            }
        };
        let Ok(code) = u32::from_str_radix(&entity[start..], radix) else {
            return Err(Error::new(entity, ErrorKind::InvalidEntityNumber));
        };
        match char::from_u32(code) {
            Some('\0'..='\x08' | '\x0a'..='\x1f' | '\x7f'..='\u{9f}') | None => {
                Err(Error::new(entity, ErrorKind::DisallowedEntityNumber))
            }
            Some(c) => Ok(c.into()),
        }
    }
}

impl<D: Decoder> Decoder for &D {
    fn get_entity<F: KeywordFilter>(&self, name: &str) -> Option<&str> {
        D::get_entity::<F>(self, name)
    }

    fn decode_entity<F>(&self, entity: &str) -> crate::Result<DecodedEntity<'_>>
    where
        F: KeywordFilter,
    {
        D::decode_entity::<F>(self, entity)
    }
}

/// Fallback `Decoder` that only looks up global entities (with [`Entity::global`]).
impl Decoder for () {
    fn get_entity<F: KeywordFilter>(&self, name: &str) -> Option<&str> {
        Entity::global(name)
    }
}

trait DecoderExt {
    fn decode<'a, K: KeywordFilter>(&self, s: &'a str) -> crate::Result<Cow<'a, str>>;

    fn decode_some<'a, K: KeywordFilter, S: AsRef<str>>(
        &self,
        s: Option<&'a S>,
    ) -> crate::Result<Option<Cow<'a, str>>> {
        match s {
            Some(s) => Ok(Some(self.decode::<K>(s.as_ref())?)),
            None => Ok(None),
        }
    }
}

impl<D: Decoder> DecoderExt for D {
    fn decode<'a, K: KeywordFilter>(&self, mut s: &'a str) -> crate::Result<Cow<'a, str>> {
        let mut res = String::new();
        while let Some((before, rest)) = s.split_once('&') {
            if !before.is_empty() {
                res.push_str(before);
            }
            let Some((entity, after)) = rest.split_once(';') else {
                return Err(Error::new(rest, ErrorKind::NoClosingSemicolon));
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

#[derive(Clone, Debug)]
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
            inner: self.inner.map(KeywordFilterIter::new),
        }
    }

    pub fn next(&mut self) -> crate::Result<Option<Cow<'a, str>>> {
        self.decoder.decode_some::<(), S>(self.inner.next())
    }

    pub fn next_or(&mut self, name: &str) -> crate::Result<Option<Cow<'a, str>>> {
        self.decoder.decode_some::<(), S>(self.inner.next_or(name))
    }

    pub fn expect_end(mut self) -> crate::Result<()> {
        if let Some(next) = self.inner.next() {
            return Err(Error::new(
                next.as_ref(),
                ErrorKind::UnexpectedEntityArguments,
            ));
        }
        Ok(())
    }
}

impl<'a, D: Decoder, S: AsRef<str>> Iterator for Scan<'a, D, S> {
    type Item = crate::Result<Cow<'a, str>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;
        Some(self.decoder.decode::<()>(next.as_ref()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

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
        self.decoder.decode_some::<K, S>(self.inner.next())
    }

    pub fn next_or(&mut self, name: &str) -> crate::Result<Option<Cow<'a, str>>> {
        self.decoder.decode_some::<K, S>(self.inner.next_or(name))
    }

    pub fn into_keywords(self) -> Result<FlagSet<K>, K::Err>
    where
        crate::Error: From<K::Err>,
    {
        self.inner.into_inner().into_keywords()
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
        Some(self.decoder.decode::<K>(next.as_ref()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
