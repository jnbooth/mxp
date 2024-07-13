use crate::color::WorldColor;
use crate::mxp::Link;
use enumeration::{Enum, EnumSet};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum TextFormat {
    Paragraph,
    Preformatted,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InList {
    Ordered(u32),
    Unordered,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Heading {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

// eg. <send "command1|command2|command3" hint="click to see menu|Item 1|Item
// 2|Item 2">this is a menu link</SEND>
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    populated: bool,
    pub(super) flags: EnumSet<TextStyle>,
    pub(super) format: EnumSet<TextFormat>,
    pub(super) foreground: Option<WorldColor>,
    pub(super) background: Option<WorldColor>,
    pub(super) action: Option<Link>,
    pub(super) list: Option<InList>,
    pub(super) heading: Option<Heading>,
    /// Which variable to set (FLAG in MXP).
    pub(super) variable: Option<String>,
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
        }
    }
}

macro_rules! set_flag {
    ($self:ident, $p:ident, $val:ident) => {
        let span = match $self.spans.get_mut(0) {
            Some(span) if !span.populated && !span.$p.contains($val) => {
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
            Some(span) if !span.populated && span.$p != $val => {
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

    pub fn get_at(&self, i: usize) -> Option<&Span> {
        self.spans.get(i)
    }

    /*
    pub fn has_format(&self, format: TextFormat) -> bool {
        match self.get() {
            Some(span) => span.format.contains(format),
            None => false,
        }
    }
    */

    pub fn truncate(&mut self, i: usize) {
        self.spans.truncate(i)
    }

    pub fn clear(&mut self) {
        self.spans.clear()
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

    pub fn set_foreground(&mut self, foreground: WorldColor) -> bool {
        set_prop!(self, foreground);
    }

    pub fn set_background(&mut self, background: WorldColor) -> bool {
        set_prop!(self, background);
    }

    pub fn set_action(&mut self, action: Link) -> bool {
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
}
