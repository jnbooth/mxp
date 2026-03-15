use std::borrow::Cow;
use std::str::FromStr;

use crate::parse::{Decoder, ExpectArg as _, Scan};

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

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Relocate<Cow<'a, str>> {
    type Error = crate::Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let hostname = scanner.next()?.expect_some("hostname")?;
        let port = scanner.next()?.expect_number()?.expect_some("port")?;
        scanner.expect_end()?;
        Ok(Self { hostname, port })
    }
}

impl FromStr for Relocate {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Relocate)
    }
}
