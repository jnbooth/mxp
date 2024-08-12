use super::argument::Keyword;
use super::error::ParseError;
use super::link::SendTo;
use crate::color::RgbColor;
use crate::entity::argument::{Arg, Argument};
use casefold::ascii::CaseFoldMap;
use enumeration::{Enum, EnumSet};
use std::borrow::{Borrow, Cow};
use std::{iter, slice, str};

pub trait Decoder {
    fn decode<'a>(&self, s: &'a str) -> Result<Cow<'a, str>, ParseError>;
}

impl Decoder for () {
    fn decode<'a>(&self, s: &'a str) -> Result<Cow<'a, str>, ParseError> {
        Ok(Cow::Borrowed(s))
    }
}

#[derive(Clone, Debug)]
pub struct Scan<'a, D> {
    pub(super) decoder: D,
    pub(super) inner: iter::Map<slice::Iter<'a, Argument>, fn(&Argument) -> &Arg>,
    pub(super) keywords: EnumSet<Keyword>,
    pub(super) named: &'a CaseFoldMap<String, Argument>,
}

impl<'a, D: Decoder> Scan<'a, D> {
    fn decode<S>(&self, s: Option<&'a S>) -> Result<Option<Cow<'a, Arg>>, ParseError>
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

    pub fn get(&self, name: &str) -> Result<Option<Cow<'a, Arg>>, ParseError> {
        self.decode(self.named.get(name))
    }

    pub fn keywords(&self) -> EnumSet<Keyword> {
        self.keywords
    }

    pub fn has_keyword(&self, keyword: Keyword) -> bool {
        self.keywords.contains(keyword)
    }

    pub fn next(&mut self) -> Result<Option<Cow<'a, Arg>>, ParseError> {
        let next = self.inner.next();
        self.decode(next)
    }

    pub fn next_or(&mut self, names: &[&str]) -> Result<Option<Cow<'a, Arg>>, ParseError> {
        match self.inner.next() {
            Some(item) => self.decoder.decode(item).map(Option::Some),
            None => self.decode(names.iter().find_map(|&name| self.named.get(name))),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AfkArgs<'a> {
    pub challenge: Option<Cow<'a, str>>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for AfkArgs<'a> {
    type Error = ParseError;

    fn try_from(mut scanner: Scan<'a, D>) -> Result<Self, ParseError> {
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
    type Error = ParseError;

    fn try_from(mut scanner: Scan<'a, D>) -> Result<Self, ParseError> {
        Ok(Self {
            fore: scanner
                .next_or(&["fore"])?
                .and_then(|fore| RgbColor::named(&fore)),
            back: scanner
                .next_or(&["back"])?
                .and_then(|back| RgbColor::named(&back)),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum FontStyle {
    Blink,
    Bold,
    Inverse,
    Italic,
    Underline,
}

impl FontStyle {
    fn parse(s: &str) -> Option<Self> {
        match_ci! {s,
            "blink" => Some(FontStyle::Blink),
            "bold" => Some(FontStyle::Bold),
            "inverse" => Some(FontStyle::Inverse),
            "italic" => Some(FontStyle::Italic),
            "underline" => Some(FontStyle::Underline),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontEffect {
    Color(RgbColor),
    Style(FontStyle),
}

impl FontEffect {
    pub fn parse(s: &str) -> Option<Self> {
        match FontStyle::parse(s) {
            Some(style) => Some(Self::Style(style)),
            None => RgbColor::named(s).map(Self::Color),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FgColor<S> {
    pub(crate) inner: S,
}

impl<S: AsRef<str>> FgColor<S> {
    pub fn iter(&self) -> impl Iterator<Item = FontEffect> + '_ {
        self.inner.as_ref().split(',').flat_map(FontEffect::parse)
    }
}

#[derive(Clone, Debug)]
pub struct FontArgs<'a> {
    pub fgcolor: FgColor<Cow<'a, str>>,
    pub bgcolor: Option<RgbColor>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for FontArgs<'a> {
    type Error = ParseError;

    fn try_from(mut scanner: Scan<'a, D>) -> Result<Self, ParseError> {
        Ok(Self {
            fgcolor: FgColor {
                inner: scanner
                    .next_or(&["color", "fgcolor"])?
                    .unwrap_or(Cow::Borrowed("")),
            },
            bgcolor: scanner
                .next_or(&["back", "bgcolor"])?
                .and_then(|bgcolor| RgbColor::named(&bgcolor)),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HyperlinkArgs<'a> {
    pub href: Option<Cow<'a, str>>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for HyperlinkArgs<'a> {
    type Error = ParseError;

    fn try_from(mut scanner: Scan<'a, D>) -> Result<Self, ParseError> {
        Ok(Self {
            href: scanner.next_or(&["href"])?,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum XchMode {
    Text,
    Html,
    PureHtml,
}

impl XchMode {
    fn parse(s: &str) -> Option<Self> {
        match_ci! {s,
            "text" => Some(Self::Text),
            "html" => Some(Self::Html),
            "purehtml" => Some(Self::PureHtml),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageArgs<'a> {
    pub fname: Option<Cow<'a, str>>,
    pub url: Option<Cow<'a, str>>,
    pub xch_mode: Option<XchMode>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for ImageArgs<'a> {
    type Error = ParseError;

    fn try_from(scanner: Scan<'a, D>) -> Result<Self, ParseError> {
        let url = match scanner.get("url")? {
            Some(url) => Some(url),
            None => scanner.get("src")?,
        };
        Ok(Self {
            fname: scanner.get("fname")?,
            url,
            xch_mode: scanner
                .get("xch_mode")?
                .and_then(|mode| XchMode::parse(&mode)),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SendArgs<'a> {
    pub href: Option<Cow<'a, str>>,
    pub hint: Option<Cow<'a, str>>,
    pub sendto: SendTo,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for SendArgs<'a> {
    type Error = ParseError;

    fn try_from(mut scanner: Scan<'a, D>) -> Result<Self, ParseError> {
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

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarArgs<'a> {
    pub variable: Option<Cow<'a, str>>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for VarArgs<'a> {
    type Error = ParseError;

    fn try_from(mut scanner: Scan<'a, D>) -> Result<Self, ParseError> {
        Ok(Self {
            variable: scanner.next()?,
        })
    }
}
