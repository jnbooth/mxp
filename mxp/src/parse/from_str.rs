use std::borrow::Cow;
use std::error::Error;
use std::fmt;

use super::scan::Scan;
use super::words::Words;
use crate::ErrorKind;
use crate::arguments::Arguments;
use crate::element::{ActionKind, Tag};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FromStrError {
    SyntaxError(crate::Error),
    UnexpectedTag(String),
}

impl From<crate::Error> for FromStrError {
    fn from(value: crate::Error) -> Self {
        Self::SyntaxError(value)
    }
}

impl fmt::Display for FromStrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SyntaxError(error) => write!(f, "syntax error: {error}"),
            Self::UnexpectedTag(tag) => write!(f, "unexpected tag: <{tag}>"),
        }
    }
}

impl Error for FromStrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SyntaxError(error) => Some(error),
            Self::UnexpectedTag(_) => None,
        }
    }
}

pub(crate) fn cleanup_source(source: &str) -> crate::Result<&str> {
    Ok(source
        .trim_ascii()
        .strip_prefix('<')
        .ok_or_else(|| crate::Error::new(source, ErrorKind::NoTagInDefinition))?
        .strip_suffix('>')
        .ok_or_else(|| crate::Error::new(source, ErrorKind::UnterminatedElement))?
        .trim_ascii())
}

pub(crate) fn parse_element<T>(source: &str, kind: ActionKind) -> Result<T, FromStrError>
where
    T: for<'a> TryFrom<Scan<'a, ()>, Error = crate::Error>,
{
    let source = cleanup_source(source)?;
    let mut words = Words::new(source);
    let name = words.next_or(ErrorKind::EmptyElement)?;
    crate::validate(name, ErrorKind::InvalidElementName)?;
    Tag::well_known(name)
        .filter(|tag| tag.action == kind)
        .ok_or_else(|| FromStrError::UnexpectedTag(name.to_owned()))?;
    let args: Arguments<Cow<str>> = words.try_into()?;
    Ok(args.scan(()).try_into()?)
}
