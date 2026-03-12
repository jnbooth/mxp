use std::borrow::Cow;

use crate::keyword::SendKeyword;
use crate::parse::{Decoder, Error, Scan};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Send<S> {
    pub href: Option<S>,
    pub hint: Option<S>,
    pub expire: Option<S>,
    pub prompt: bool,
}

impl<'a, D> TryFrom<Scan<'a, D>> for Send<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        let href = scanner.next_or("href")?;
        let hint = scanner.next_or("hint")?;
        let expire = scanner.next_or("expire")?;
        let keywords = scanner.into_keywords()?;
        Ok(Self {
            href,
            hint,
            expire,
            prompt: keywords.contains(SendKeyword::Prompt),
        })
    }
}
