use std::borrow::Cow;

use crate::argument::{Decoder, ExpectArg, Scan};
use crate::parser::Error;

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

impl Relocate<&str> {
    pub fn into_owned(self) -> Relocate<String> {
        Relocate {
            hostname: self.hostname.to_owned(),
            port: self.port,
        }
    }
}

impl Relocate<Cow<'_, str>> {
    pub fn into_owned(self) -> Relocate<String> {
        Relocate {
            hostname: self.hostname.into_owned(),
            port: self.port,
        }
    }
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for Relocate<Cow<'a, str>>
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        Ok(Self {
            hostname: scanner.next()?.expect_some("hostname")?,
            port: scanner.next()?.expect_number()?.expect_some("port")?,
        })
    }
}
