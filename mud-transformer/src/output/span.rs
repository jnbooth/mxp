#![allow(clippy::redundant_field_names)]
use std::ops::Index;

use super::color::TermColor;
use enumeration::{Enum, EnumSet};
use mxp::escape::ansi;
use mxp::Heading;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum TextStyle {
    Blink,
    Bold,
    Highlight,
    Inverse,
    Italic,
    Strikeout,
    Underline,
}

impl TextStyle {
    pub const fn ansi(self) -> Option<u8> {
        match self {
            Self::Blink => Some(ansi::BLINK),
            Self::Bold => Some(ansi::BOLD),
            Self::Highlight => None,
            Self::Inverse => Some(ansi::INVERSE),
            Self::Italic => Some(ansi::SLOW_BLINK),
            Self::Strikeout => Some(ansi::STRIKEOUT),
            Self::Underline => Some(ansi::UNDERLINE),
        }
    }
}

impl From<mxp::FontStyle> for TextStyle {
    fn from(value: mxp::FontStyle) -> Self {
        match value {
            mxp::FontStyle::Blink => Self::Blink,
            mxp::FontStyle::Bold => Self::Bold,
            mxp::FontStyle::Inverse => Self::Inverse,
            mxp::FontStyle::Italic => Self::Italic,
            mxp::FontStyle::Underline => Self::Underline,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum TextFormat {
    Paragraph,
    Pre,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InList {
    Ordered(u32),
    Unordered,
}

/// eg. <send "command1|command2|command3" hint="click to see menu|Item 1|Item
/// 2|Item 2">this is a menu link</SEND>
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    populated: bool,
    pub(super) flags: EnumSet<TextStyle>,
    pub(super) format: EnumSet<TextFormat>,
    pub(super) foreground: Option<TermColor>,
    pub(super) background: Option<TermColor>,
    pub(super) action: Option<mxp::Link>,
    pub(super) list: Option<InList>,
    pub(super) heading: Option<Heading>,
    /// Which variable to set (FLAG in MXP).
    pub(super) variable: Option<String>,
    pub(super) variable_flags: EnumSet<mxp::EntityKeyword>,
    pub(super) gag: bool,
    pub(super) window: Option<String>,
}

impl Default for Span {
    fn default() -> Self {
        Self::new()
    }
}

impl Span {
    pub const fn new() -> Self {
        Self {
            populated: false,
            flags: EnumSet::new(),
            format: EnumSet::new(),
            foreground: None,
            background: None,
            action: None,
            list: None,
            heading: None,
            variable: None,
            variable_flags: EnumSet::new(),
            gag: false,
            window: None,
        }
    }
}

macro_rules! set_flag {
    ($self:ident, $p:ident, $val:ident) => {
        let span = match $self.get_mut() {
            Some(span) if span.$p.contains($val) => {
                return false;
            }
            Some(span) if !span.populated => {
                span.$p.insert($val);
                return false;
            }
            Some(span) => Span {
                populated: false,
                $p: span.$p | $val,
                ..span.clone()
            },
            None => Span {
                $p: enums![$val],
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpanList {
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

    pub fn format(&self) -> EnumSet<TextFormat> {
        match self.get() {
            Some(span) => span.format,
            None => EnumSet::new(),
        }
    }

    pub fn truncate(&mut self, i: usize) {
        self.spans.truncate(i);
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

    pub fn next_list_item(&mut self) -> Option<u32> {
        match self.get_mut()?.list.as_mut()? {
            InList::Unordered => Some(0),
            InList::Ordered(i) => {
                *i += 1;
                Some(*i)
            }
        }
    }

    pub fn set_flag(&mut self, flag: TextStyle) -> bool {
        set_flag!(self, flags, flag);
    }

    pub fn set_format(&mut self, format: TextFormat) -> bool {
        set_flag!(self, format, format);
    }

    pub fn unset_format(&mut self, format: TextFormat) -> bool {
        let Some(span) = self.get_mut() else {
            return false;
        };
        if !span.format.contains(format) {
            return false;
        }
        if !span.populated {
            span.format.remove(format);
            return false;
        }
        let mut format_flags = span.format;
        format_flags.remove(format);
        let next_span = Span {
            populated: false,
            format: format_flags,
            ..span.clone()
        };
        self.spans.push(next_span);
        true
    }

    pub fn set_foreground(&mut self, foreground: TermColor) -> bool {
        set_prop!(self, foreground);
    }

    pub fn set_background(&mut self, background: TermColor) -> bool {
        set_prop!(self, background);
    }

    pub fn set_action(&mut self, action: mxp::Link) -> bool {
        set_prop!(self, action);
    }

    pub fn set_list(&mut self, list: InList) -> bool {
        set_prop!(self, list);
    }

    pub fn set_heading(&mut self, heading: Heading) -> bool {
        set_prop!(self, heading);
    }

    pub fn set_variable(&mut self, variable: String) -> bool {
        set_prop!(self, variable);
    }

    pub fn set_variable_flags(&mut self, variable_flags: EnumSet<mxp::EntityKeyword>) -> bool {
        set_prop!(self, variable_flags, variable_flags);
    }

    pub fn set_gag(&mut self) -> bool {
        set_prop!(self, gag, true);
    }

    pub fn set_window(&mut self, window: String) -> bool {
        set_prop!(self, window);
    }
}
