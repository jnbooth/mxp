use std::borrow::Cow;

use crate::argument::{Decoder, ExpectArg as _, Scan};
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

impl<S> Relocate<S> {
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

impl<'a, D> TryFrom<Scan<'a, D>> for Relocate<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            hostname: scanner.next()?.expect_some("hostname")?,
            port: scanner.next()?.expect_number()?.expect_some("port")?,
        })
    }
}
