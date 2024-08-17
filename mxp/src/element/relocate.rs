use crate::argument::{Decoder, ExpectArg, Scan};
use crate::parser::Error;
use std::borrow::Cow;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl<'a> Relocate<Cow<'a, str>> {
    pub fn into_owned(self) -> Relocate {
        Relocate {
            hostname: self.hostname.into_owned(),
            port: self.port,
        }
    }
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for Relocate<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            hostname: scanner.next()?.expect_some("hostname")?,
            port: scanner.next()?.expect_number()?.expect_some("port")?,
        })
    }
}
