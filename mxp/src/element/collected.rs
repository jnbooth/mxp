use std::str::FromStr;

use crate::parse::{Error, ErrorKind, StringVariant, UnrecognizedVariant, Words};

/// Type of MXP definition sent by the server.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DefinitionKind {
    /// [`<!ATTLIST>`](https://www.zuggsoft.com/zmud/mxp.htm#ATTLIST):
    /// Add attributes to elements.
    AttributeList,
    /// [`<!ELEMENT>`](https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT):
    /// Define a new [`Element`](crate::Element).
    Element,
    /// [`<!ENTITY>`](https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT):
    /// Define a new [`Entity`](crate::Entity).
    Entity,
    /// [`<!TAG>`](https://www.zuggsoft.com/zmud/mxp.htm#User-defined%20Line%20Tags):
    /// Change properties for a line tag.
    LineTag,
}

impl StringVariant for DefinitionKind {
    type Variant = &'static str;
    const VARIANTS: &[&str] = &["ATTLIST", "ATT", "ELEMENT", "EL", "ENTITY", "EN", "TAG"];
}

impl FromStr for DefinitionKind {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "attlist" | "att" => Ok(Self::AttributeList),
            "element" | "el" => Ok(Self::Element),
            "entity" | "en" => Ok(Self::Entity),
            "tag" => Ok(Self::LineTag),
            _ => Err(Self::Err::new(s))
        }
    }
}
/// MXP definition sent by the server, which may define an [attribute list], [element], [entity],
/// or [line tag].
///
/// [attribute list]: https://www.zuggsoft.com/zmud/mxp.htm#ATTLIST
/// [element]: https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT
/// [entity]: https://www.zuggsoft.com/zmud/mxp.htm#ENTITY
/// [line tag]: https://www.zuggsoft.com/zmud/mxp.htm#User-defined%20Line%20Tags
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CollectedDefinition<'a> {
    pub(crate) kind: DefinitionKind,
    pub(crate) text: &'a str,
}

impl<'a> CollectedDefinition<'a> {
    fn parse(text: &'a str) -> Option<Self> {
        let (kind, definition) = text.split_once(' ')?;
        Some(Self {
            kind: kind.parse().ok()?,
            text: definition,
        })
    }
}

/// The three types of MXP tag elements sent by the server.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CollectedElement<'a> {
    /// A definition, e.g. [`<!ELEMENT>`].
    Definition(CollectedDefinition<'a>),
    /// A closing tag, e.g. [`</BOLD>`].
    TagClose(&'a str),
    /// An opening tag, e.g. [`<BOLD>`].
    TagOpen(&'a str),
}

impl<'a> CollectedElement<'a> {
    pub(crate) fn parse(text: &'a str) -> crate::Result<Self> {
        let tag = *text
            .as_bytes()
            .first()
            .ok_or_else(|| Error::new(text, ErrorKind::EmptyElement))?;

        match tag {
            b'!' => {
                let body = &text[1..];
                if body.is_empty() {
                    return Err(Error::new(text, ErrorKind::ElementTooShort));
                }
                let definition = CollectedDefinition::parse(body)
                    .ok_or_else(|| Error::new(body, ErrorKind::InvalidDefinition))?;
                Ok(Self::Definition(definition))
            }
            b'/' => {
                let body = &text[1..];
                if body.is_empty() {
                    return Err(Error::new(text, ErrorKind::ElementTooShort));
                }
                let mut words = Words::new(body);
                let name = words.validate_next_or(ErrorKind::InvalidElementName)?;
                if words.next().is_some() {
                    return Err(Error::new(body, ErrorKind::ArgumentsToClosingTag));
                }
                Ok(Self::TagClose(name))
            }
            _ => Ok(Self::TagOpen(text)),
        }
    }
}
