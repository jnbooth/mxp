use std::fmt;

use crate::arguments::{ArgumentScanner, FromArgs};

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

impl<'a, S: AsRef<str>> FromArgs<'a, S> for Expire<S> {
    fn from_args<A: ArgumentScanner<'a, Decoded = S>>(mut scanner: A) -> crate::Result<Self> {
        let name = scanner.get_next()?;
        scanner.expect_end()?;
        Ok(Self { name })
    }
}

impl_from_str!(Expire);

impl<S: AsRef<str>> fmt::Display for Expire<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Expire { name } = self.borrow_text();
        crate::display::ElementFormatter {
            name: "EXPIRE",
            arguments: &[&name],
            keywords: &[],
        }
        .fmt(f)
    }
}
