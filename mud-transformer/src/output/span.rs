#![allow(clippy::redundant_field_names)]
use std::num::NonZero;
use std::ops::Index;

use bytes::BytesMut;
use bytestring::ByteString;
use flagset::{FlagSet, flags};
use mxp::Heading;
use mxp::escape::ansi;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EntitySetter {
    pub name: String,
    pub flags: FlagSet<mxp::EntityKeyword>,
    pub is_variable: bool,
}

/// eg. <send "command1|command2|command3" hint="click to see menu|Item 1|Item
/// 2|Item 2">this is a menu link</SEND>
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Span {
    populated: bool,
    pub(super) flags: FlagSet<TextStyle>,
    pub(super) foreground: TermColor,
    pub(super) background: TermColor,
    pub(super) font: Option<ByteString>,
    pub(super) size: Option<NonZero<u8>>,
    pub(super) action: Option<mxp::Link>,
    pub(super) heading: Option<Heading>,
    pub(super) gag: bool,
    pub(super) window: Option<ByteString>,
    pub(super) entity: Option<EntitySetter>,
}

macro_rules! set_flag {
    ($self:ident, $p:ident, $val:ident) => {
        let span = match $self.spans.as_mut_slice().last_mut() {
            Some(span) if span.$p.contains($val) => {
                return false;
            }
            Some(span) if !span.populated => {
                span.$p |= $val;
                return false;
            }
            Some(span) => Span {
                populated: false,
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
    ($self:ident, $p:ident) => {
        let span = match $self.spans.as_mut_slice().last_mut() {
            Some(span) if span.$p == $p => {
                return false;
            }
            Some(span) if !span.populated => {
                span.$p = $p;
                return false;
            }
            Some(span) => Span {
                populated: false,
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
    ($self:ident, $p:ident) => {
        set_opt_prop!($self, $p, $p)
    };
    ($self:ident, $p:ident, $val:expr) => {
        let span = match $self.spans.as_mut_slice().last_mut() {
            Some(Span {
                $p: Some(other), ..
            }) if other == &$p => {
                return false;
            }
            Some(span) if !span.populated => {
                span.$p = Some($val);
                return false;
            }
            Some(span) => Span {
                populated: false,
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
    ($self:ident, $p:ident) => {
        set_opt_prop!($self, $p, share_string(&mut $self.buf, $p));
    };
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SpanList {
    spans: Vec<Span>,
    buf: BytesMut,
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

    const fn get_mut(&mut self) -> Option<&mut Span> {
        self.spans.as_mut_slice().last_mut()
    }

    pub fn truncate(&mut self, i: usize) -> Option<EntitySetter> {
        if i >= self.spans.len() {
            return None;
        }
        let entity = self.spans.pop().and_then(|span| span.entity);
        self.spans.truncate(i);
        entity
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

    pub fn set_populated(&mut self) {
        if let Some(span) = self.get_mut() {
            span.populated = true;
        }
    }

    pub fn set_flag(&mut self, flag: TextStyle) -> bool {
        set_flag!(self, flags, flag);
    }

    pub fn set_foreground(&mut self, foreground: TermColor) -> bool {
        set_prop!(self, foreground);
    }

    pub fn set_background(&mut self, background: TermColor) -> bool {
        set_prop!(self, background);
    }

    pub fn set_font(&mut self, font: &str) -> bool {
        set_string_prop!(self, font);
    }

    pub fn set_size(&mut self, size: NonZero<u8>) -> bool {
        set_opt_prop!(self, size);
    }

    pub fn set_action(&mut self, action: mxp::Link) -> bool {
        set_opt_prop!(self, action);
    }

    pub fn set_heading(&mut self, heading: Heading) -> bool {
        set_opt_prop!(self, heading);
    }

    pub fn set_entity(&mut self, entity: EntitySetter) -> bool {
        set_opt_prop!(self, entity);
    }

    pub fn set_gag(&mut self) -> bool {
        let gag = true;
        set_prop!(self, gag);
    }

    pub fn set_window(&mut self, window: &str) -> bool {
        set_string_prop!(self, window);
    }
}

fn share_string(buf: &mut BytesMut, s: &str) -> ByteString {
    buf.extend_from_slice(s.as_bytes());
    let bytes = buf.split().freeze();
    // SAFETY: `bytes` is valid UTF-8.
    unsafe { ByteString::from_bytes_unchecked(bytes) }
}
