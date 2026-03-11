use std::borrow::Cow;
use std::str::FromStr;

use crate::parse::{Decoder, Error, Scan};

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

impl<'a, D> TryFrom<Scan<'a, D>> for Support<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> Result<Self, Self::Error> {
        let mut questions = Vec::with_capacity(scanner.len());
        while let Some(question) = scanner.next()? {
            questions.push(question);
        }
        Ok(Self { questions })
    }
}

impl FromStr for Support {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Support)
    }
}
