use std::borrow::Cow;

use crate::arguments::{ArgumentScanner, ExpectArg as _};
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
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Relocate<S = String> {
    /// Hostname of the new connection.
    pub hostname: S,
    /// Port of the new connection.
    pub port: u16,
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
        }
    }
}

impl_partial_eq!(Relocate);

impl<S: AsRef<str>> Relocate<S> {
    pub(crate) fn scan<A>(mut scanner: A) -> crate::Result<Self>
    where
        A: ArgumentScanner<Output = S>,
    {
        let hostname = scanner.decode_next()?.expect_some("Hostname")?;
        let port = scanner
            .decode_next()?
            .expect_number()?
            .expect_some("Port")?;
        scanner.expect_end()?;
        Ok(Self { hostname, port })
    }
}

impl_from_str!(Relocate);
