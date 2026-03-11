use std::borrow::Cow;

use crate::parse::{Decoder, Error, ExpectArg as _, Scan};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Hyperlink<S> {
    pub href: S,
    pub hint: Option<S>,
    pub expire: Option<S>,
}

impl<'a, D> TryFrom<Scan<'a, D>> for Hyperlink<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            href: scanner.next_or("href")?.expect_some("href")?,
            hint: scanner.next_or("hint")?,
            expire: scanner.next_or("expire")?,
        })
    }
}
