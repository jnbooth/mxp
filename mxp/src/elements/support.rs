use std::borrow::Cow;
use std::{fmt, slice, vec};

use flagset::FlagSet;

use crate::arguments::ArgumentScanner;
use crate::element::ActionKind;
use crate::parse::Decoder;
use crate::responses::SupportResponse;

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

    /// Alias for `self.questions.iter()`.
    pub fn iter(&self) -> slice::Iter<'_, S> {
        self.questions.iter()
    }

    /// Constructs a `SupportResponse` from this struct's questions.
    pub fn respond(&self, supported: FlagSet<ActionKind>) -> SupportResponse<slice::Iter<'_, S>>
    where
        S: AsRef<str>,
    {
        SupportResponse::new(self.questions.iter(), supported)
    }
}

impl_into_owned!(Support);

impl<S> IntoIterator for Support<S> {
    type Item = S;

    type IntoIter = vec::IntoIter<S>;

    fn into_iter(self) -> Self::IntoIter {
        self.questions.into_iter()
    }
}

impl<'a, S> IntoIterator for &'a Support<S> {
    type Item = &'a S;

    type IntoIter = slice::Iter<'a, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.questions.iter()
    }
}

impl<S: AsRef<str>> Support<S> {
    pub(crate) fn scan<A>(mut scanner: A) -> crate::Result<Self>
    where
        A: ArgumentScanner<Output = S>,
    {
        let mut questions = Vec::new();
        while let Some(question) = scanner.decode_next()? {
            questions.push(question);
        }
        Ok(Self { questions })
    }
}

impl_from_str!(Support);

impl<S: AsRef<str>> fmt::Display for Support<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Support { questions } = self;
        if questions.is_empty() {
            return f.write_str("<SUPPORT>");
        }
        f.write_str("<SUPPORT")?;
        for question in questions {
            write!(f, " \"{}\"", question.as_ref())?;
        }
        f.write_str(">")
    }
}
