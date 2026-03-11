use std::borrow::Cow;
use std::str::FromStr;

use crate::parse::{Decoder, Error, ExpectArg as _, Scan};

/// Defines a graphics format and provides a client plugin module that converts the MUD-specific
/// format to a standard GIF or BMP format.
///
/// See [MXP specification: `<FILTER>`](https://www.zuggsoft.com/zmud/mxp.htm#File%20Filters).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter<S = String> {
    /// File extension of the MUD-specific format.
    pub src: S,
    pub dest: S,
    pub name: S,
}

impl<S> Filter<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, mut f: F) -> Filter<T>
    where
        F: FnMut(S) -> T,
    {
        Filter {
            src: f(self.src),
            dest: f(self.dest),
            name: f(self.name),
        }
    }
}

impl_into_owned!(Filter);

impl<S: AsRef<str>> Filter<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Filter<&str> {
        Filter {
            src: self.src.as_ref(),
            dest: self.dest.as_ref(),
            name: self.name.as_ref(),
        }
    }
}

impl_partial_eq!(Filter);

impl<'a, D> TryFrom<Scan<'a, D>> for Filter<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            src: scanner.next_or("src")?.expect_some("src")?,
            dest: scanner.next_or("dest")?.expect_some("dest")?,
            name: scanner.next_or("name")?.expect_some("name")?,
        })
    }
}

impl FromStr for Filter {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Filter)
    }
}
