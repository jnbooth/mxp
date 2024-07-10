use super::link::Link;
use enumeration::{Enum, EnumSet};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InList {
    Ordered(u32),
    Unordered,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum TextStyle {
    Underline,
    Italic,
    Strikeout,
    Inverse,
    Bold,
}

// eg. <send "command1|command2|command3" hint="click to see menu|Item 1|Item
// 2|Item 2">this is a menu link</SEND>
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub flags: EnumSet<TextStyle>,
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub action: Option<Link>,
    /// Which variable to set (FLAG in MXP).
    pub variable: Option<String>,
    pub list: Option<InList>,
}

impl Span {
    pub fn child(&self) -> Self {
        Self {
            flags: self.flags,
            foreground: None,
            background: None,
            action: None,
            variable: None,
            list: self.list,
        }
    }
}
