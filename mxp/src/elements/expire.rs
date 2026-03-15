use std::borrow::Cow;
use std::str::FromStr;

use crate::parse::{Decoder, Scan};

/// Removes previously displayed links. For example, when moving to a new room, links from the
/// previous room description are no longer valid and need to be removed.
///
/// See [MXP specification: `<EXPIRE>`](https://www.zuggsoft.com/zmud/mxp.htm#Links).
///
/// # Examples
///
/// ```
/// assert_eq!(
///     "<EXPIRE exits>".parse::<mxp::Expire>(),
///     Ok(mxp::Expire {
///         name: Some("exits".into()),
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Expire<S = String> {
    /// Name of the link to remove. If `None`, expire all links with names.
    pub name: Option<S>,
}

impl<S> Expire<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, f: F) -> Expire<T>
    where
        F: FnOnce(S) -> T,
    {
        Expire {
            name: self.name.map(f),
        }
    }
}

impl_into_owned!(Expire);

impl<S: AsRef<str>> Expire<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Expire<&str> {
        Expire {
            name: self.name.as_ref().map(AsRef::as_ref),
        }
    }
}

impl_partial_eq!(Expire);

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Expire<Cow<'a, str>> {
    type Error = crate::Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let name = scanner.next()?;
        scanner.expect_end()?;
        Ok(Self { name })
    }
}

impl FromStr for Expire {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Expire)
    }
}
