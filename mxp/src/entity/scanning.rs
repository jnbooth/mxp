use super::argument::{Arguments, Keyword};
use super::link::SendTo;
use crate::color::RgbColor;
use enumeration::Enum;
use std::str;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AfkArgs<'a> {
    pub challenge: Option<&'a str>,
}

impl<'a> From<&'a Arguments> for AfkArgs<'a> {
    fn from(args: &'a Arguments) -> Self {
        let mut scanner = args.scan();
        Self {
            challenge: scanner.next_or(&["challenge"]),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorArgs {
    pub fore: Option<RgbColor>,
    pub back: Option<RgbColor>,
}

impl From<&Arguments> for ColorArgs {
    fn from(args: &Arguments) -> Self {
        let mut scanner = args.scan();
        Self {
            fore: scanner.next_or(&["fore"]).and_then(RgbColor::named),
            back: scanner.next_or(&["back"]).and_then(RgbColor::named),
        }
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
    pub fgcolor: FgColor<&'a str>,
    pub bgcolor: Option<RgbColor>,
}

impl<'a> From<&'a Arguments> for FontArgs<'a> {
    fn from(args: &'a Arguments) -> Self {
        let mut scanner = args.scan();

        Self {
            fgcolor: FgColor {
                inner: scanner.next_or(&["color", "fgcolor"]).unwrap_or(""),
            },
            bgcolor: scanner
                .next_or(&["back", "bgcolor"])
                .and_then(RgbColor::named),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HyperlinkArgs<'a> {
    pub href: Option<&'a str>,
}

impl<'a> From<&'a Arguments> for HyperlinkArgs<'a> {
    fn from(args: &'a Arguments) -> Self {
        let mut scanner = args.scan();
        Self {
            href: scanner.next_or(&["href"]),
        }
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageArgs<'a> {
    pub fname: Option<&'a str>,
    pub url: Option<&'a str>,
    pub xch_mode: Option<XchMode>,
}

impl<'a> From<&'a Arguments> for ImageArgs<'a> {
    fn from(args: &'a Arguments) -> Self {
        Self {
            fname: args.get("fname"),
            url: args.get("url").or_else(|| args.get("src")),
            xch_mode: args.get("xch_mode").and_then(XchMode::parse),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SendArgs<'a> {
    pub href: Option<&'a str>,
    pub hint: Option<&'a str>,
    pub sendto: SendTo,
}

impl<'a> From<&'a Arguments> for SendArgs<'a> {
    fn from(args: &'a Arguments) -> Self {
        let mut scanner = args.scan();
        Self {
            href: scanner.next_or(&["href", "xch_cmd"]),
            hint: scanner.next_or(&["hint", "xch_hint"]),
            sendto: if args.has_keyword(Keyword::Prompt) {
                SendTo::Input
            } else {
                SendTo::World
            },
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarArgs<'a> {
    pub variable: Option<&'a str>,
}

impl<'a> From<&'a Arguments> for VarArgs<'a> {
    fn from(args: &'a Arguments) -> Self {
        Self {
            variable: args.get(0),
        }
    }
}
