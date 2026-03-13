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

/// The three types of MXP tag elements sent by the server.
#[derive(Clone, Debug)]
pub enum CollectedElement<'a> {
    /// A definition, e.g. [`<!ELEMENT>`].
    Definition(DefinitionKind, &'a str),
    /// A closing tag, e.g. [`</BOLD>`].
    TagClose(&'a str),
    /// An opening tag, e.g. [`<BOLD>`].
    TagOpen(&'a str),
}

impl<'a> CollectedElement<'a> {
    pub(crate) fn parse(source: &'a str, secure: bool) -> crate::Result<Self> {
        let source = source.trim_ascii();

        match source.split_at_checked(1) {
            None if source.is_empty() => Err(Error::new(source, ErrorKind::EmptyElement)),
            Some(("!" | "/", "")) => Err(Error::new(source, ErrorKind::ElementTooShort)),
            Some(("!", _)) if !secure => {
                Err(Error::new(source, ErrorKind::DefinitionWhenNotSecure))
            }
            Some(("!", body)) => {
                let mut words = Words::new(body);
                let kind = words
                    .next()
                    .and_then(|kind| kind.parse().ok())
                    .ok_or_else(|| Error::new(body, ErrorKind::InvalidDefinition))?;
                Ok(Self::Definition(kind, words.as_str()))
            }
            Some(("/", body)) => {
                let mut words = Words::new(body);
                let name = words.validate_next_or(ErrorKind::InvalidElementName)?;
                if words.next().is_some() {
                    return Err(Error::new(body, ErrorKind::ArgumentsToClosingTag));
                }
                Ok(Self::TagClose(name))
            }
            _ => Ok(Self::TagOpen(source)),
        }
    }
}
