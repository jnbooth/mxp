use std::borrow::Cow;
use std::fmt;

use crate::arguments::{ArgumentScanner, ExpectArg as _};
use crate::keyword::RelocateKeyword;
use crate::parse::Decoder;

/// Closes the current MUD connection and causes a new connect to open on a new server.
///
/// See [MXP specification: `<RELOCATE>`](https://www.zuggsoft.com/zmud/mxp.htm#Crosslinking%20multiple%20MUD%20servers).
///
/// # Examples
///
/// ```
/// assert_eq!(
///     "<RELOCATE new.server.com 1000>".parse::<mxp::Relocate>(),
///     Ok(mxp::Relocate {
///         hostname: "new.server.com".into(),
///         port: 1000,
///         quiet: false,
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Relocate<S = String> {
    /// Hostname of the new connection.
    pub hostname: S,
    /// Port of the new connection.
    pub port: u16,
    /// The optional keyword QUIET can be used to suppress further output from the MUD. When the closing </RELOCATE> tag is used, MUD output is resumed.
    pub quiet: bool,
}

impl<S> Relocate<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, f: F) -> Relocate<T>
    where
        F: FnOnce(S) -> T,
    {
        Relocate {
            hostname: f(self.hostname),
            port: self.port,
            quiet: self.quiet,
        }
    }
}

impl_into_owned!(Relocate);

impl<S: AsRef<str>> Relocate<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Relocate<&str> {
        Relocate {
            hostname: self.hostname.as_ref(),
            port: self.port,
            quiet: self.quiet,
        }
    }
}

impl_partial_eq!(Relocate);

impl<S: AsRef<str>> Relocate<S> {
    pub(crate) fn scan<A>(scanner: A) -> crate::Result<Self>
    where
        A: ArgumentScanner<Output = S>,
    {
        let mut scanner = scanner.with_keywords();
        let hostname = scanner.decode_next()?.expect_some("Hostname")?;
        let port = scanner
            .decode_next()?
            .expect_number()?
            .expect_some("Port")?;
        let keywords = scanner.into_keywords()?;
        Ok(Self {
            hostname,
            port,
            quiet: keywords.contains(RelocateKeyword::Quiet),
        })
    }
}

impl_from_str!(Relocate);

impl<S: AsRef<str>> fmt::Display for Relocate<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Relocate {
            hostname,
            port,
            quiet,
        } = self.borrow_text();
        crate::display::ElementFormatter {
            name: "RELOCATE",
            arguments: &[&hostname, &port],
            keywords: &[("QUIET", quiet)],
        }
        .fmt(f)
    }
}
