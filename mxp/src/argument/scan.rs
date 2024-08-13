use super::font_args::FgColor;
use super::keyword::Keyword;
use super::pueblo::XchMode;
use crate::color::RgbColor;
use crate::entity::SendTo;
use crate::parser::Error;
use casefold::ascii::CaseFoldMap;
use enumeration::EnumSet;
use std::borrow::Borrow;
use std::{iter, slice, str};

pub trait Decoder {
    type Output<'a>: AsRef<str>;

    fn decode<'a>(&self, s: &'a str) -> crate::Result<Self::Output<'a>>;
}

impl<D: Decoder> Decoder for &D {
    type Output<'a> = D::Output<'a>;

    fn decode<'a>(&self, s: &'a str) -> crate::Result<Self::Output<'a>> {
        D::decode(self, s)
    }
}

#[derive(Clone, Debug)]
pub struct Scan<'a, D> {
    pub(super) decoder: D,
    pub(super) inner: iter::Map<slice::Iter<'a, String>, fn(&String) -> &str>,
    pub(super) keywords: EnumSet<Keyword>,
    pub(super) named: &'a CaseFoldMap<String, String>,
}

impl<'a, D: Decoder> Scan<'a, D> {
    fn decode<S>(&self, s: Option<&'a S>) -> crate::Result<Option<D::Output<'a>>>
    where
        S: Borrow<str> + ?Sized,
    {
        match s {
            Some(s) => self.decoder.decode(s.borrow()).map(Option::Some),
            None => Ok(None),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn get(&self, name: &str) -> crate::Result<Option<D::Output<'a>>> {
        self.decode(self.named.get(name))
    }

    pub fn keywords(&self) -> EnumSet<Keyword> {
        self.keywords
    }

    pub fn has_keyword(&self, keyword: Keyword) -> bool {
        self.keywords.contains(keyword)
    }

    pub fn next(&mut self) -> crate::Result<Option<D::Output<'a>>> {
        let next = self.inner.next();
        self.decode(next)
    }

    pub fn next_or(&mut self, names: &[&str]) -> crate::Result<Option<D::Output<'a>>> {
        match self.inner.next() {
            Some(item) => self.decoder.decode(item).map(Option::Some),
            None => self.decode(names.iter().find_map(|&name| self.named.get(name))),
        }
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
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for ImageArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let url = match scanner.get("url")? {
            Some(url) => Some(url),
            None => scanner.get("src")?,
        };
        Ok(Self {
            fname: scanner.get("fname")?,
            url,
            xch_mode: scanner
                .get("xch_mode")?
                .and_then(|mode| XchMode::parse(mode.as_ref())),
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

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            href: scanner.next_or(&["href", "xch_cmd"])?,
            hint: scanner.next_or(&["hint", "xch_hint"])?,
            sendto: if scanner.has_keyword(Keyword::Prompt) {
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
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for VarArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            variable: scanner.next()?,
        })
    }
}
