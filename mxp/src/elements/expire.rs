use std::borrow::Cow;

use crate::Error;
use crate::argument::{Decoder, Scan};

/// [`<EXPIRE>`](https://www.zuggsoft.com/zmud/mxp.htm#Links):
/// Expire links.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Expire<S = String> {
    pub name: Option<S>,
}

impl<S> Expire<S> {
    pub fn map_text<T, F>(self, f: F) -> Expire<T>
    where
        F: FnOnce(S) -> T,
    {
        Expire {
            name: self.name.map(f),
        }
    }
}

impl_into_owned!(Expire);

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
