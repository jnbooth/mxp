use std::fmt;

/// An MXP entity, either standard or server-defined.
///
/// See [MXP specification: Entities](https://www.zuggsoft.com/zmud/mxp.htm#ENTITY).
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DecodedEntity<'a> {
    /// A standard XML entity, such as `&amp;` or `&#032;`.
    Standard(char),
    /// A custom MXP entity defined by the server.
    Custom(&'a str),
}

impl DecodedEntity<'_> {
    /// Appends the decoded entity to a string.
    #[inline]
    pub fn push_to(self, buf: &mut String) {
        match self {
            Self::Standard(c) => buf.push(c),
            Self::Custom(s) => buf.push_str(s),
        }
    }
}

impl From<char> for DecodedEntity<'_> {
    #[inline]
    fn from(value: char) -> Self {
        Self::Standard(value)
    }
}

impl<'a> From<&'a str> for DecodedEntity<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::Custom(value)
    }
}

impl fmt::Debug for DecodedEntity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Standard(c) => c.fmt(f),
            Self::Custom(s) => s.fmt(f),
        }
    }
}

impl fmt::Display for DecodedEntity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Standard(c) => c.fmt(f),
            Self::Custom(s) => s.fmt(f),
        }
    }
}
