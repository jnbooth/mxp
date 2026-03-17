use std::fmt;

use super::definition::Definition;
use crate::arguments::Arguments;
use crate::parse::Words;
use crate::{Error, ErrorKind};

/// The three types of MXP tag elements sent by the server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tag<'a> {
    /// A definition, e.g. `<!ELEMENT...>`.
    Definition(Definition<'a>),
    /// A closing tag, e.g. `</BOLD>`.
    Close(TagClose<'a>),
    /// An opening tag, e.g. `<BOLD>`.
    Open(TagOpen<'a>),
}

impl fmt::Display for Tag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Definition(def) => def.fmt(f),
            Self::Close(tag) => tag.fmt(f),
            Self::Open(tag) => tag.fmt(f),
        }
    }
}

impl<'a> Tag<'a> {
    /// Returns the name of the element.
    pub fn name(&self) -> &'a str {
        match self {
            Self::Definition(definition) => definition.name(),
            Self::Close(tag) => tag.name,
            Self::Open(tag) => tag.name,
        }
    }

    /// Parses an element from data sent by the server.
    ///
    /// Returns an error if `secure` is false and the data is a definition tag (`<!...>`).
    /// Definitions can only be processed if the current line mode is not OPEN. See
    /// [`Mode::is_open`] for more.
    ///
    /// Important note: this function expects `source` to omit the starting `<` and ending `>`.
    ///
    /// [`Mode::is_open`]: crate::Mode::is_open
    pub fn parse(source: &'a str, secure: bool) -> crate::Result<Self> {
        let source = source.trim_ascii();

        match source.split_at_checked(1) {
            None if source.is_empty() => Err(Error::braced("", ErrorKind::EmptyElement)),
            Some(("!" | "/", "")) => Err(Error::braced(source, ErrorKind::IncompleteElement)),
            Some(("!", _)) if !secure => Err(Error::braced(source, ErrorKind::UnsecuredDefinition)),
            Some(("!", body)) => Ok(Self::Definition(Definition::parse(body)?)),
            Some(("/", body)) => Ok(Self::Close(TagClose::parse(body)?)),
            _ => Ok(Self::Open(TagOpen::parse(source)?)),
        }
    }
}

/// Parsed representation of a closing tag from the server, in the form of `</{name}>`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TagClose<'a> {
    /// Element name.
    pub name: &'a str,
}

impl fmt::Display for TagClose<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { name } = self;
        write!(f, "</{name}>")
    }
}

impl<'a> TagClose<'a> {
    fn parse(source: &'a str) -> crate::Result<Self> {
        let mut words = Words::new(source);
        let name = words.next_or(ErrorKind::IncompleteElement)?;
        crate::validate(name, ErrorKind::InvalidElementName)?;
        if let Some(next) = words.next() {
            return Err(Error::new(next, ErrorKind::ArgumentsToClosingTag));
        }
        Ok(Self { name })
    }
}

/// Parsed representation of an opening tag from the server, in the form of `<{name} ...>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagOpen<'a> {
    /// Element name.
    pub name: &'a str,
    /// Parsed element arguments.
    pub arguments: Arguments<'a>,
}

impl fmt::Display for TagOpen<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { name, arguments } = self;
        write!(f, "<{name} {arguments}>")
    }
}

impl<'a> TagOpen<'a> {
    fn parse(source: &'a str) -> crate::Result<Self> {
        let mut words = Words::new(source);
        let name = words.next_or(ErrorKind::EmptyElement)?;
        crate::validate(name, ErrorKind::InvalidElementName)?;
        Ok(Self {
            name,
            arguments: words.parse_args()?,
        })
    }
}
