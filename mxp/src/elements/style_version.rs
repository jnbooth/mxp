use std::borrow::Cow;
use std::str::FromStr;

use crate::arguments::{ArgumentScanner, ExpectArg as _};
use crate::parse::{Decoder, Scan};

/// A MUD sets a style-sheet version number by sending the `<VERSION styleversion>` tag to the
/// client.
///
/// The client caches this version information and returns it when requested by a plain
/// `<VERSION>` request.
///
/// See [MXP specification: `<VERSION>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control).
///
/// # Examples
///
/// ```
/// assert_eq!(
///     "<VERSION 0.6>".parse::<mxp::StyleVersion>(),
///     Ok(mxp::StyleVersion {
///         styleversion: "0.6".into(),
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct StyleVersion<S = String> {
    /// Style-sheet version number.
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

impl<S: AsRef<str>> StyleVersion<S> {
    pub(crate) fn scan<A>(mut scanner: A) -> crate::Result<Self>
    where
        A: ArgumentScanner<Output = S>,
    {
        let styleversion = scanner.next()?.expect_some("StyleVersion")?;
        scanner.expect_end()?;
        Ok(Self { styleversion })
    }
}

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for StyleVersion<Cow<'a, str>> {
    type Error = crate::Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        Self::scan(scanner)
    }
}

impl FromStr for StyleVersion {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Version)
    }
}
