use std::borrow::Cow;

use crate::Error;
use crate::argument::{Decoder, Scan};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Expire<S = String> {
    pub name: Option<S>,
}

impl Expire<&str> {
    pub fn into_owned(self) -> Expire<String> {
        Expire {
            name: self.name.map(ToOwned::to_owned),
        }
    }
}

impl Expire<Cow<'_, str>> {
    pub fn into_owned(self) -> Expire<String> {
        Expire {
            name: self.name.map(Cow::into_owned),
        }
    }
}

impl<'a, D> TryFrom<Scan<'a, D>> for Expire<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            name: scanner.next()?,
        })
    }
}
