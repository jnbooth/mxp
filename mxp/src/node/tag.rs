use std::borrow::{Borrow, Cow};
use std::fmt;

use html_escape::encode_double_quoted_attribute as escape;

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
pub struct TagOpen<'a, S = &'a str> {
    /// Element name.
    pub name: &'a str,
    /// Parsed element arguments.
    pub arguments: Arguments<'a, S>,
}

impl<S: AsRef<str>> fmt::Display for TagOpen<'_, S> {
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

/// Utility for servers to build custom opening tags.
///
/// # Examples
///
/// ```
/// let mut builder = mxp::node::TagBuilder::new("monster");
/// builder
///     .push("hostile")
///     .push_all(&["Minotaur", "veteran"])
///     .insert("hp", "150");
/// let tag = builder.build();
/// assert_eq!(
///     tag.to_string(),
///     r#"<monster "hostile" "Minotaur" "veteran" hp="150">"#
/// );
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagBuilder<'a> {
    tag: TagOpen<'a, Cow<'a, str>>,
}

/// A server-side utility for building opening tags.
///
///
impl<'a> TagBuilder<'a> {
    /// Creates a builder for an opening tag with the specified element name.
    pub fn new(name: &'a str) -> Self {
        Self {
            tag: TagOpen {
                name,
                arguments: Arguments::default(),
            },
        }
    }

    /// Converts this builder into a finalized `TagOpen`.
    pub fn build(self) -> TagOpen<'a, Cow<'a, str>> {
        self.tag
    }

    /// Adds a positional argument or keyword without escaping special characters.
    pub fn push(&mut self, value: &'a str) -> &mut Self {
        self.tag.arguments.push(value.into());
        self
    }

    /// Adds a positional argument or keyword after escaping special characters.
    pub fn escape_and_push(&mut self, value: &'a str) -> &mut Self {
        self.tag.arguments.push(escape(value));
        self
    }

    /// Adds multiple positional arguments or keywords without escaping special characters.
    pub fn push_all<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Borrow<&'a str>,
    {
        self.tag
            .arguments
            .extend(iter.into_iter().map(|s| Cow::Borrowed(*s.borrow())));
        self
    }

    /// Adds multiple positional arguments or keywords after escaping special characters.
    pub fn escape_and_push_all<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Borrow<&'a str>,
    {
        self.tag
            .arguments
            .extend(iter.into_iter().map(|s| escape(*s.borrow())));
        self
    }

    /// Inserts a named argument without escaping special characters.
    pub fn insert(&mut self, name: &'a str, value: &'a str) -> &mut Self {
        self.tag.arguments.insert(name, value.into());
        self
    }

    /// Inserts a named argument after escaping special characters.
    pub fn escape_and_insert(&mut self, name: &'a str, value: &'a str) -> &mut Self {
        self.tag.arguments.insert(name, escape(value));
        self
    }

    /// Inserts multiple named arguments without escaping special characters.
    pub fn insert_all<I, K, V>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: Borrow<&'a str>,
        V: Borrow<&'a str>,
    {
        self.tag.arguments.extend(iter.into_iter().map(|entry| {
            let (k, v) = entry.borrow();
            (*k.borrow(), Cow::Borrowed(*v.borrow()))
        }));
        self
    }

    /// Inserts multiple named arguments after escaping special characters.
    pub fn escape_and_insert_all<I, K, V>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: Borrow<&'a str>,
        V: Borrow<&'a str>,
    {
        self.tag.arguments.extend(iter.into_iter().map(|entry| {
            let (k, v) = entry.borrow();
            (*k.borrow(), escape(*v.borrow()))
        }));
        self
    }
}
