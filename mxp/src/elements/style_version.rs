use std::borrow::Cow;
use std::str::FromStr;

use crate::Error;
use crate::parse::{Decoder, ExpectArg as _, Scan};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct StyleVersion<S = String> {
    pub styleversion: S,
}

impl<S> StyleVersion<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, f: F) -> StyleVersion<T>
    where
        F: FnOnce(S) -> T,
    {
        StyleVersion {
            styleversion: f(self.styleversion),
        }
    }
}

impl_into_owned!(StyleVersion);

impl<S: AsRef<str>> StyleVersion<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> StyleVersion<&str> {
        StyleVersion {
            styleversion: self.styleversion.as_ref(),
        }
    }
}

impl_partial_eq!(StyleVersion);

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for StyleVersion<Cow<'a, str>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            styleversion: scanner.next()?.expect_some("styleversion")?,
        })
    }
}

impl FromStr for StyleVersion {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Version)
    }
}
