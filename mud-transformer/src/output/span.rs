#![allow(clippy::redundant_field_names)]
use std::num::NonZero;
use std::ops::Index;

use flagset::{FlagSet, flags};
use mxp::Heading;
use mxp::escape::ansi;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::color::TermColor;

flags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialOrd, Ord, Hash)]
    pub enum TextStyle: u16 {
        Blink,
        Bold,
        Highlight,
        Italic,
        NonProportional,
        Small,
        Strikeout,
        Underline,
        Inverse,
    }
}

impl TextStyle {
    pub const fn ansi(self) -> Option<u8> {
        match self {
            Self::Blink => Some(ansi::BLINK),
            Self::Bold => Some(ansi::BOLD),
            Self::Italic => Some(ansi::SLOW_BLINK),
            Self::Strikeout => Some(ansi::STRIKEOUT),
            Self::Underline => Some(ansi::UNDERLINE),
            Self::Inverse => Some(ansi::INVERSE),
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
    pub(super) foreground: Option<TermColor>,
    pub(super) background: Option<TermColor>,
    pub(super) font: Option<String>,
    pub(super) size: Option<NonZero<u8>>,
    pub(super) action: Option<mxp::Link>,
    pub(super) heading: Option<Heading>,
    pub(super) gag: bool,
    pub(super) window: Option<String>,
    pub(super) entity: Option<EntitySetter>,
}

macro_rules! set_flag {
    ($self:ident, $p:ident, $val:ident) => {
        let span = match $self.get_mut() {
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
        let some_val = Some($p);
        set_prop!($self, $p, some_val)
    };
    ($self:ident, $p:ident, $val:expr) => {
        let span = match $self.get_mut() {
            Some(span) if span.$p == $val => {
                return false;
            }
            Some(span) if !span.populated => {
                span.$p = $val;
                return false;
            }
            Some(span) => Span {
                populated: false,
                $p: $val,
                ..span.clone()
            },
            None => Span {
                $p: $val,
                ..Default::default()
            },
        };
        $self.spans.push(span);
        return true;
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpanList {
    spans: Vec<Span>,
}

impl Default for SpanList {
    fn default() -> Self {
        Self::new()
    }
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
    pub const fn new() -> Self {
        Self { spans: Vec::new() }
    }

    pub fn get(&self) -> Option<&Span> {
        self.spans.last()
    }

    fn get_mut(&mut self) -> Option<&mut Span> {
        self.spans.last_mut()
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

    pub fn len(&self) -> usize {
        self.spans.len()
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

    pub fn set_font(&mut self, font: String) -> bool {
        set_prop!(self, font);
    }

    pub fn set_size(&mut self, size: NonZero<u8>) -> bool {
        set_prop!(self, size);
    }

    pub fn set_action(&mut self, action: mxp::Link) -> bool {
        set_prop!(self, action);
    }

    pub fn set_heading(&mut self, heading: Heading) -> bool {
        set_prop!(self, heading);
    }

    pub fn set_entity(&mut self, entity: EntitySetter) -> bool {
        set_prop!(self, entity);
    }

    pub fn set_gag(&mut self) -> bool {
        set_prop!(self, gag, true);
    }

    pub fn set_window(&mut self, window: String) -> bool {
        set_prop!(self, window);
    }
}
