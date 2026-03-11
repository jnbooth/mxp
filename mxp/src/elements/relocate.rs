use std::borrow::Cow;
use std::str::FromStr;

use crate::parse::{Decoder, Error, ExpectArg as _, Scan};

/// Closes the current MUD connection and causes a new connect to open on a new server.
///
/// See [MXP specification: `<RELOCATE>`](https://www.zuggsoft.com/zmud/mxp.htm#Crosslinking%20multiple%20MUD%20servers).
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

impl<'a, D> TryFrom<Scan<'a, D>> for Relocate<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
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
