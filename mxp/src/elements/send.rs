use std::borrow::Cow;

use super::link::SendTo;
use crate::argument::{Decoder, Scan};
use crate::keyword::SendKeyword;
use crate::parser::Error;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Send<S> {
    pub href: Option<S>,
    pub hint: Option<S>,
    pub send_to: SendTo,
    pub expire: Option<S>,
}

impl<'a, D> TryFrom<Scan<'a, D>> for Send<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        Ok(Self {
            href: scanner.next_or("href")?,
            hint: scanner.next_or("hint")?,
            expire: scanner.next_or("expire")?,
            send_to: if scanner.into_keywords().contains(SendKeyword::Prompt) {
                SendTo::Input
            } else {
                SendTo::World
            },
        })
    }
}
