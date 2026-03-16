use super::arguments_str::ArgumentsStr;
use super::definition::ParsedDefinition;
use crate::parse::Words;
use crate::{Error, ErrorKind};

/// The three types of MXP tag elements sent by the server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParsedElement<'a> {
    /// A definition, e.g. `<!ELEMENT...>`.
    Definition(ParsedDefinition<'a>),
    /// A closing tag, e.g. `</BOLD>`.
    TagClose(ParsedTagClose<'a>),
    /// An opening tag, e.g. `<BOLD>`.
    TagOpen(ParsedTagOpen<'a>),
}

impl<'a> ParsedElement<'a> {
    /// Returns the name of the element.
    pub fn name(&self) -> &'a str {
        match self {
            Self::Definition(definition) => definition.name(),
            Self::TagClose(tag) => tag.name,
            Self::TagOpen(tag) => tag.name,
        }
    }

    /// Parses an element from data sent by the server.
    ///
    /// Returns an error if `secure` is false and the data is a definition tag (`<!...>`).
    /// Definitions can only be processed if the current line mode is
    /// [secure](crate::Mode::is_secure).
    pub fn parse(source: &'a str, secure: bool) -> crate::Result<Self> {
        let source = source.trim_ascii();

        match source.split_at_checked(1) {
            None if source.is_empty() => Err(Error::braced("", ErrorKind::EmptyElement)),
            Some(("!" | "/", "")) => Err(Error::braced(source, ErrorKind::IncompleteElement)),
            Some(("!", _)) if !secure => Err(Error::braced(source, ErrorKind::UnsecuredDefinition)),
            Some(("!", body)) => Ok(Self::Definition(ParsedDefinition::parse(body)?)),
            Some(("/", body)) => Ok(Self::TagClose(ParsedTagClose::parse(body)?)),
            _ => Ok(Self::TagOpen(ParsedTagOpen::parse(source)?)),
        }
    }
}

/// Parsed representation of a closing tag from the server, in the form of `</{name}>`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ParsedTagClose<'a> {
    /// Element name.
    pub name: &'a str,
}

impl<'a> ParsedTagClose<'a> {
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ParsedTagOpen<'a> {
    /// Element name.
    pub name: &'a str,
    /// The rest of the definition as a string slice. This should be parsed with
    /// [`arguments.parse_args()`] for use with functions such as [`AtomicTag::decode`] and
    /// [`Element::decode`].
    ///
    /// [`arguments.parse_args()`]: ArgumentsStr::parse_args
    /// [`AtomicTag::decode`]: crate::AtomicTag::decode
    /// [`Element::decode`]: crate::Element::decode
    pub arguments: ArgumentsStr<'a>,
}

impl<'a> ParsedTagOpen<'a> {
    fn parse(source: &'a str) -> crate::Result<Self> {
        let mut words = Words::new(source);
        let name = words.next_or(ErrorKind::EmptyElement)?;
        crate::validate(name, ErrorKind::InvalidElementName)?;
        Ok(Self {
            name,
            arguments: ArgumentsStr(words.as_str()),
        })
    }
}
