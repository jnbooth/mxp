#![allow(clippy::redundant_field_names)]
use std::num::NonZero;
use std::ops::Index;

use bytestring::ByteString;
use bytestringmut::ByteStringMut;
use flagset::{FlagSet, flags};
use mxp::Heading;
use mxp::escape::ansi;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::bytestringmut_ext::ByteStringMutExt as _;
use super::link::Link;
use crate::term::TermColor;

flags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum TextStyle: u16 {
        NonProportional,
        Bold,
        Faint,
        Italic,
        Underline,
        Blink,
        Small,
        Inverse,
        Conceal,
        Strikeout,
        Highlight,
    }
}

impl TextStyle {
    pub const fn ansi(self) -> Option<u8> {
        match self {
            Self::Bold => Some(ansi::BOLD),
            Self::Faint => Some(ansi::FAINT),
            Self::Italic => Some(ansi::ITALIC),
            Self::Underline => Some(ansi::UNDERLINE),
            Self::Blink => Some(ansi::SLOW_BLINK),
            Self::Inverse => Some(ansi::INVERSE),
            Self::Conceal => Some(ansi::CONCEAL),
            Self::Strikeout => Some(ansi::STRIKEOUT),
            Self::Highlight | Self::NonProportional | Self::Small => None,
        }
    }
}

impl From<mxp::FontStyle> for TextStyle {
    fn from(value: mxp::FontStyle) -> Self {
        match value {
            mxp::FontStyle::Blink => Self::Blink,
            mxp::FontStyle::Bold => Self::Bold,
            mxp::FontStyle::Italic => Self::Italic,
            mxp::FontStyle::Underline => Self::Underline,
            mxp::FontStyle::Inverse => Self::Inverse,
        }
    }
}

/// eg. <send "command1|command2|command3" hint="click to see menu|Item 1|Item
/// 2|Item 2">this is a menu link</SEND>
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Span {
    pub(super) flags: FlagSet<TextStyle>,
    pub(super) foreground: TermColor,
    pub(super) background: TermColor,
    pub(super) font: Option<ByteString>,
    pub(super) size: Option<NonZero<u8>>,
    pub(super) link: Option<Link>,
    pub(super) heading: Option<Heading>,
    pub(super) gag: bool,
    pub(super) window: Option<mxp::Dest<ByteString>>,
    pub(super) entity: Option<mxp::Var<ByteString>>,
    pub(super) variable: Option<ByteString>,
    pub(super) parse_as: Option<mxp::ParseAs>,
}

macro_rules! set_flag {
    ($self:ident, $empty:expr, $p:ident, $val:ident) => {
        let span = match $self.spans.as_mut_slice().last_mut() {
            Some(span) if span.$p.contains($val) => {
                return false;
            }
            Some(span) if $empty => {
                span.$p |= $val;
                return false;
            }
            Some(span) => Span {
                $p: span.$p | $val,
                ..span.clone()
            },
            None => Span {
                $p: $val.into(),
                ..Default::default()
            },
        };
        $self.spans.push(span);
        #[allow(clippy::needless_return)]
        return true;
    };
}

macro_rules! set_prop {
    ($self:ident, $empty:expr, $p:ident) => {
        let span = match $self.spans.as_mut_slice().last_mut() {
            Some(span) if span.$p == $p => {
                return false;
            }
            Some(span) if $empty => {
                span.$p = $p;
                return false;
            }
            Some(span) => Span {
                $p: $p,
                ..span.clone()
            },
            None => Span {
                $p: $p,
                ..Default::default()
            },
        };
        $self.spans.push(span);
        return true;
    };
}

macro_rules! set_opt_prop {
    ($self:ident, $empty:expr, $p:ident) => {
        set_opt_prop!($self, $empty, $p, $p)
    };
    ($self:ident, $empty:expr, $p:ident, $val:expr) => {
        let span = match $self.spans.as_mut_slice().last_mut() {
            Some(Span {
                $p: Some(other), ..
            }) if other == &$p => {
                return false;
            }
            Some(span) if $empty => {
                span.$p = Some($val);
                return false;
            }
            Some(span) => Span {
                $p: Some($val),
                ..span.clone()
            },
            None => Span {
                $p: Some($val),
                ..Default::default()
            },
        };
        $self.spans.push(span);
        return true;
    };
}

macro_rules! set_string_prop {
    ($self:ident, $empty:expr, $p:ident) => {
        set_opt_prop!($self, $empty, $p, $self.buf.share($p));
    };
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SpanList {
    spans: Vec<Span>,
    buf: ByteStringMut,
}

impl<I> Index<I> for SpanList
where
    Vec<Span>: Index<I, Output = Span>,
{
    type Output = Span;

    fn index(&self, index: I) -> &Self::Output {
        self.spans.index(index)
    }
}

impl SpanList {
    pub const fn get(&self) -> Option<&Span> {
        self.spans.as_slice().last()
    }

    pub fn truncate(&mut self, i: usize) -> Option<Span> {
        if i >= self.spans.len() {
            return None;
        }
        let span = self.spans.pop()?;
        self.spans.truncate(i);
        Some(span)
    }

    pub fn clear(&mut self) {
        self.spans.clear();
    }

    pub const fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn reset_ansi(&mut self) {
        for span in &mut self.spans {
            span.foreground = TermColor::Unset;
            span.background = TermColor::Unset;
            span.flags.clear();
        }
    }

    pub fn set_background(&mut self, background: TermColor, empty: bool) -> bool {
        set_prop!(self, empty, background);
    }

    pub fn set_entity<S: AsRef<str>>(&mut self, entity: mxp::Var<S>, empty: bool) -> bool {
        let entity = entity.map_text(|text| self.buf.share(text.as_ref()));
        set_opt_prop!(self, empty, entity);
    }

    pub fn set_flag(&mut self, flag: TextStyle, empty: bool) -> bool {
        set_flag!(self, empty, flags, flag);
    }

    pub fn set_foreground(&mut self, foreground: TermColor, empty: bool) -> bool {
        set_prop!(self, empty, foreground);
    }

    pub fn set_font(&mut self, font: &str, empty: bool) -> bool {
        set_string_prop!(self, empty, font);
    }

    pub fn set_gag(&mut self, empty: bool) -> bool {
        let gag = true;
        set_prop!(self, empty, gag);
    }

    pub fn set_heading(&mut self, heading: Heading, empty: bool) -> bool {
        set_opt_prop!(self, empty, heading);
    }

    pub fn set_link(&mut self, link: Link, empty: bool) -> bool {
        set_opt_prop!(self, empty, link);
    }

    pub fn set_parse_as(&mut self, parse_as: mxp::ParseAs, empty: bool) -> bool {
        set_opt_prop!(self, empty, parse_as);
    }

    pub fn set_size(&mut self, size: NonZero<u8>, empty: bool) -> bool {
        set_opt_prop!(self, empty, size);
    }

    pub fn set_window<S: AsRef<str>>(&mut self, window: mxp::Dest<S>, empty: bool) -> bool {
        let window = window.map_text(|text| self.buf.share(text.as_ref()));
        set_opt_prop!(self, empty, window);
    }

    pub fn set_variable(&mut self, variable: &str, empty: bool) -> bool {
        set_string_prop!(self, empty, variable);
    }
}
