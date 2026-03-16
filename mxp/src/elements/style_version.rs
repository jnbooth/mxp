use std::borrow::Cow;

use crate::arguments::{ArgumentScanner, ExpectArg as _};
use crate::parse::Decoder;

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
        let styleversion = scanner.decode_next()?.expect_some("StyleVersion")?;
        scanner.expect_end()?;
        Ok(Self { styleversion })
    }
}

impl_from_str!(StyleVersion);
