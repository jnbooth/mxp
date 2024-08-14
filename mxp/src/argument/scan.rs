use super::arguments::{KeywordFilter, NoKeywords};
use crate::color::RgbColor;
use crate::entity::{Atom, Link, SendTo};
use crate::keyword::{EntityKeyword, MxpKeyword, SendKeyword};
use crate::parser::{Error, ErrorKind};
use casefold::ascii::CaseFoldMap;
use enumeration::{Enum, EnumSet};
use std::borrow::Borrow;
use std::marker::PhantomData;
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

    pub fn get(&self, name: &str) -> crate::Result<Option<D::Output<'a>>> {
        self.decode(self.named.get(name))
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
pub struct ExpireArgs<S> {
    pub name: Option<S>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for ExpireArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            name: scanner.next()?,
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HyperlinkArgs<S> {
    pub href: S,
    pub hint: Option<S>,
    pub expire: Option<S>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for HyperlinkArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            href: scanner
                .next_or("href")?
                .ok_or_else(|| Error::new("", ErrorKind::NoArgument))?,
            hint: scanner.next_or("hint")?,
            expire: scanner.next_or("expire")?,
        })
    }
}

impl<S: AsRef<str>> From<HyperlinkArgs<S>> for Link {
    fn from(value: HyperlinkArgs<S>) -> Self {
        Self::new(
            value.href.as_ref(),
            value.hint.as_ref().map(AsRef::as_ref),
            SendTo::Internet,
            value.expire.map(|expire| expire.as_ref().to_owned()),
        )
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SendArgs<S> {
    pub href: Option<S>,
    pub hint: Option<S>,
    pub sendto: SendTo,
    pub expire: Option<S>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for SendArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        Ok(Self {
            href: scanner.next_or("href")?,
            hint: scanner.next_or("hint")?,
            expire: scanner.next_or("expire")?,
            sendto: if scanner.into_keywords().contains(SendKeyword::Prompt) {
                SendTo::Input
            } else {
                SendTo::World
            },
        })
    }
}

impl<S: AsRef<str>> From<SendArgs<S>> for Link {
    fn from(value: SendArgs<S>) -> Self {
        Self::new(
            value
                .href
                .as_ref()
                .map_or(Link::EMBED_ENTITY, AsRef::as_ref),
            value.hint.as_ref().map(AsRef::as_ref),
            value.sendto,
            value.expire.map(|expire| expire.as_ref().to_owned()),
        )
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
                .ok_or_else(|| Error::new("", ErrorKind::NoArgument))?,
            keywords: scanner.into_keywords(),
        })
    }
}
