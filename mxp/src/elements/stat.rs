use std::borrow::Cow;

use crate::argument::{Decoder, ExpectArg as _, Scan};
use crate::parser::Error;

/// Displays an MXP entity value as status bar text.
///
/// See [MXP specification: `<STAT>`](https://www.zuggsoft.com/zmud/mxp.htm#Using%20Entities).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Stat<S = String> {
    /// Name of the entity to use as text data.
    pub entity: S,
    /// Name of the entity to use for the maximum value of the data.
    pub max: Option<S>,
    /// Optional caption text.
    pub caption: Option<S>,
}

impl Stat<&str> {
    pub fn into_owned(self) -> Stat<String> {
        Stat {
            entity: self.entity.to_owned(),
            max: self.max.map(ToOwned::to_owned),
            caption: self.caption.map(ToOwned::to_owned),
        }
    }
}

impl Stat<Cow<'_, str>> {
    pub fn into_owned(self) -> Stat<String> {
        Stat {
            entity: self.entity.into_owned(),
            max: self.max.map(Cow::into_owned),
            caption: self.caption.map(Cow::into_owned),
        }
    }
}

impl<'a, D> TryFrom<Scan<'a, D>> for Stat<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            entity: scanner.next()?.expect_some("EntityName")?,
            max: scanner.next_or("max")?,
            caption: scanner.next_or("caption")?,
        })
    }
}
