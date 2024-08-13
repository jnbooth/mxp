use super::arguments::{KeywordFilter, NoKeywords};
use super::font_args::FgColor;
use super::pueblo::XchMode;
use crate::color::RgbColor;
use crate::entity::SendTo;
use crate::keyword::{EntityKeyword, ImageKeyword, SendKeyword};
use crate::parser::Error;
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

    fn find_by_names(&self, names: &[&str]) -> crate::Result<Option<D::Output<'a>>> {
        self.decode(names.iter().find_map(|&name| self.named.get(name)))
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

    pub fn next_or(&mut self, names: &[&str]) -> crate::Result<Option<D::Output<'a>>> {
        match self.next()? {
            Some(item) => Ok(Some(item)),
            None => self.find_by_names(names),
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

    pub fn next_or(&mut self, names: &[&str]) -> crate::Result<Option<D::Output<'a>>> {
        match self.next()? {
            Some(item) => Ok(Some(item)),
            None => self.find_by_names(names),
        }
    }

    pub fn into_keywords(self) -> EnumSet<K> {
        let mut keywords = self.keywords;
        for keyword in self.inner.inner.flat_map(|arg| arg.parse().ok()) {
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
            challenge: scanner.next_or(&["challenge"])?,
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
                .next_or(&["fore"])?
                .and_then(|fore| RgbColor::named(fore.as_ref())),
            back: scanner
                .next_or(&["back"])?
                .and_then(|back| RgbColor::named(back.as_ref())),
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FontArgs<S> {
    pub fgcolor: Option<FgColor<S>>,
    pub bgcolor: Option<RgbColor>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for FontArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            fgcolor: scanner
                .next_or(&["color", "fgcolor"])?
                .map(|fgcolor| FgColor { inner: fgcolor }),
            bgcolor: scanner
                .next_or(&["back", "bgcolor"])?
                .and_then(|bgcolor| RgbColor::named(bgcolor.as_ref())),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HyperlinkArgs<S> {
    pub href: Option<S>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for HyperlinkArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            href: scanner.next_or(&["href"])?,
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageArgs<S> {
    pub fname: Option<S>,
    pub url: Option<S>,
    pub xch_mode: Option<XchMode>,
    pub is_map: bool,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for ImageArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let scanner = scanner.with_keywords();
        let url = match scanner.get("url")? {
            Some(url) => Some(url),
            None => scanner.get("src")?,
        };
        let fname = scanner.get("fname")?;
        let xch_mode = scanner
            .get("xch_mode")?
            .and_then(|mode| mode.as_ref().parse().ok());
        let keywords = scanner.into_keywords();
        Ok(Self {
            fname,
            url,
            xch_mode,
            is_map: keywords.contains(ImageKeyword::IsMap),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SendArgs<S> {
    pub href: Option<S>,
    pub hint: Option<S>,
    pub sendto: SendTo,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for SendArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        Ok(Self {
            href: scanner.next_or(&["href", "xch_cmd"])?,
            hint: scanner.next_or(&["hint", "xch_hint"])?,
            sendto: if scanner.into_keywords().contains(SendKeyword::Prompt) {
                SendTo::Input
            } else {
                SendTo::World
            },
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarArgs<S> {
    pub variable: Option<S>,
    pub keywords: EnumSet<EntityKeyword>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for VarArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        Ok(Self {
            variable: scanner.next()?,
            keywords: scanner.into_keywords(),
        })
    }
}
