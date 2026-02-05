use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SelectionData {
    Clipboard,
    Primary,
    Secondary,
    Selection,
    CutBuffer(u8),
}

impl SelectionData {
    pub(crate) const fn from_code(code: u8) -> Option<Self> {
        match code {
            b'c' => Some(Self::Clipboard),
            b'p' => Some(Self::Primary),
            b'q' => Some(Self::Secondary),
            b's' => Some(Self::Selection),
            (b'0'..=b'9') => Some(Self::CutBuffer(code - b'0')),
            _ => None,
        }
    }
}

impl fmt::Display for SelectionData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Clipboard => f.write_str("c"),
            Self::Primary => f.write_str("p"),
            Self::Secondary => f.write_str("q"),
            Self::Selection => f.write_str("s"),
            Self::CutBuffer(n) => write!(f, "{n}"),
        }
    }
}
