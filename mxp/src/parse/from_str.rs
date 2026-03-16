use std::error::Error;
use std::fmt;

use super::words::Words;
use crate::ErrorKind;
use crate::arguments::Arguments;
use crate::element::{ActionKind, Tag};
use crate::parse::OwnedScan;

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

pub(crate) trait HasActionKind {
    const ACTION_KIND: ActionKind;
}

macro_rules! impl_kind {
    ($t:ident) => {
        impl<S> HasActionKind for crate::elements::$t<S> {
            const ACTION_KIND: ActionKind = ActionKind::$t;
        }
    };
}

impl_kind!(Dest);
impl_kind!(Expire);
impl_kind!(Filter);
impl_kind!(Font);
impl_kind!(Frame);
impl_kind!(Gauge);
impl_kind!(Hyperlink);
impl_kind!(Image);
impl_kind!(Music);
impl_kind!(Relocate);
impl_kind!(Send);
impl_kind!(Sound);
impl_kind!(Support);
impl_kind!(Stat);
impl_kind!(Var);

impl HasActionKind for crate::elements::Color {
    const ACTION_KIND: ActionKind = ActionKind::Color;
}

impl<S> HasActionKind for crate::elements::StyleVersion<S> {
    const ACTION_KIND: ActionKind = ActionKind::Version;
}

pub(crate) fn parse_element<'a, T>(source: &'a str) -> Result<T, FromStrError>
where
    T: HasActionKind + TryFrom<OwnedScan<'a, ()>, Error = crate::Error>,
{
    let source = cleanup_source(source)?;
    let mut words = Words::new(source);
    let name = words.next_or(ErrorKind::EmptyElement)?;
    crate::validate(name, ErrorKind::InvalidElementName)?;
    Tag::well_known(name)
        .filter(|tag| tag.action == T::ACTION_KIND)
        .ok_or_else(|| FromStrError::UnexpectedTag(name.to_owned()))?;
    let args: Arguments = words.try_into()?;
    Ok(args.into_scan(()).try_into()?)
}
