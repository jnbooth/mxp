use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DecodedEntity<'a> {
    Char(char),
    Str(&'a str),
}

impl DecodedEntity<'_> {
    #[inline]
    pub fn push_to(self, buf: &mut String) {
        match self {
            Self::Char(c) => buf.push(c),
            Self::Str(s) => buf.push_str(s),
        }
    }
}

impl From<char> for DecodedEntity<'_> {
    #[inline]
    fn from(value: char) -> Self {
        Self::Char(value)
    }
}

impl<'a> From<&'a str> for DecodedEntity<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::Str(value)
    }
}

impl fmt::Debug for DecodedEntity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Char(c) => c.fmt(f),
            Self::Str(s) => s.fmt(f),
        }
    }
}

impl fmt::Display for DecodedEntity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Char(c) => c.fmt(f),
            Self::Str(s) => s.fmt(f),
        }
    }
}
