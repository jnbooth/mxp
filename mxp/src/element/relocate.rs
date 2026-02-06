use std::borrow::Cow;

use crate::argument::{Decoder, ExpectArg, Scan};
use crate::parser::Error;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Relocate<S = String> {
    pub hostname: S,
    pub port: u16,
}

impl Relocate<&str> {
    pub fn into_owned(self) -> Relocate {
        Relocate {
            hostname: self.hostname.to_owned(),
            port: self.port,
        }
    }
}

impl Relocate<Cow<'_, str>> {
    pub fn into_owned(self) -> Relocate {
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
