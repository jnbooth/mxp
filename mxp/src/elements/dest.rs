use std::borrow::Cow;

use crate::Error;
use crate::argument::{Decoder, ExpectArg as _, Scan};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Dest<S = String> {
    pub name: S,
}

impl Dest<&str> {
    pub fn into_owned(self) -> Dest<String> {
        Dest {
            name: self.name.to_owned(),
        }
    }
}

impl Dest<Cow<'_, str>> {
    pub fn into_owned(self) -> Dest<String> {
        Dest {
            name: self.name.into_owned(),
        }
    }
}

impl<'a, D> TryFrom<Scan<'a, D>> for Dest<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            name: scanner.next()?.expect_some("name")?,
        })
    }
}
