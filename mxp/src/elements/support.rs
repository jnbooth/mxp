use std::borrow::Cow;
use std::str::FromStr;

use crate::parse::{Decoder, Error, Scan};

/// Determines exactly which tags are supported by the client.
///
/// See [MXP specification: `<SUPPORT>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control).
///
/// # Examples
///
/// ```
/// assert_eq!(
///     "<SUPPORT 'color.*' send.expire image>".parse::<mxp::Support>(),
///     Ok(mxp::Support {
///         questions: vec!["color.*".into(), "send.expire".into(), "image".into()],
///     }),
/// );
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Support<S = String> {
    pub questions: Vec<S>,
}

impl<S> Support<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, f: F) -> Support<T>
    where
        F: FnMut(S) -> T,
    {
        Support {
            questions: self.questions.into_iter().map(f).collect(),
        }
    }
}

impl_into_owned!(Support);

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Support<Cow<'a, str>> {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        Ok(Self {
            questions: scanner.collect::<Result<_, _>>()?,
        })
    }
}

impl FromStr for Support {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Support)
    }
}
