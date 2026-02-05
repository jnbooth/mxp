use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SelectionData {
    Clipboard,
    Primary,
    Select,
    CutBuffer(u8),
}

impl SelectionData {
    pub(crate) const fn from_code(code: u8) -> Option<Self> {
        match code {
            b'c' => Some(Self::Clipboard),
            b'p' => Some(Self::Primary),
            b's' => Some(Self::Select),
            (b'0'..=b'7') => Some(Self::CutBuffer(code - b'0')),
            _ => None,
        }
    }
}

impl fmt::Display for SelectionData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Clipboard => f.write_str("c"),
            Self::Primary => f.write_str("p"),
            Self::Select => f.write_str("s"),
            Self::CutBuffer(n) => write!(f, "{n}"),
        }
    }
}
