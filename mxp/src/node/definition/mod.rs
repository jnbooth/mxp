use std::fmt;
use std::str::FromStr;

use super::error::TryFromNodeError;
use crate::parse::split_name;
use crate::{Error, ErrorKind};

mod attribute_list;
pub use attribute_list::AttributeListDefinition;

mod element;
pub use element::ElementDefinition;

mod entity;
pub use entity::EntityDefinition;

mod line_tag;
pub use line_tag::LineTagDefinition;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum DefinitionKind {
    AttributeList,
    Element,
    Entity,
    LineTag,
}

impl FromStr for DefinitionKind {
    type Err = Error;

    fn from_str(s: &str) -> crate::Result<Self> {
        match_ci! {s,
            "attlist" | "at" => Ok(Self::AttributeList),
            "element" | "el" => Ok(Self::Element),
            "entity" | "en" => Ok(Self::Entity),
            "tag" => Ok(Self::LineTag),
            _ => Err(Error::new(s, ErrorKind::InvalidDefinition))
        }
    }
}

/// Syntax tree of a definition tag from the server, in the form of `<!...>`.
///
/// Note: This is the parameter type of [`State::define`](crate::State::define).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Definition<'a> {
    /// `<!ATTLIST ...>` or `<!AT ...>`.
    AttributeList(AttributeListDefinition<'a>),
    /// `<!ELEMENT ...>` or `<!EL ...>`.
    Element(ElementDefinition<'a>),
    /// `<!ENTITY ...>` or `<!EN ...>`.
    Entity(EntityDefinition<'a>),
    /// `<!TAG ...>`.
    LineTag(LineTagDefinition<'a>),
}

impl fmt::Display for Definition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AttributeList(def) => def.fmt(f),
            Self::Element(def) => def.fmt(f),
            Self::Entity(def) => def.fmt(f),
            Self::LineTag(def) => def.fmt(f),
        }
    }
}

impl<'a> Definition<'a> {
    /// Returns the name of the item being defined.
    ///
    /// Note: A [`LineTagDefinition`] does not contain a name, so it will return `""` instead.
    pub fn name(&self) -> &'a str {
        match self {
            Self::AttributeList(def) => def.name,
            Self::Element(def) => def.name,
            Self::Entity(def) => def.name,
            Self::LineTag(_) => "",
        }
    }

    pub(super) fn parse(source: &'a str) -> crate::Result<Self> {
        let (name, rest) = split_name(source);
        if name.is_empty() {
            return Err(Error::new("empty definition", ErrorKind::IncompleteElement));
        }
        let kind = name.parse()?;
        Ok(match kind {
            DefinitionKind::AttributeList => {
                Self::AttributeList(AttributeListDefinition::parse(rest)?)
            }
            DefinitionKind::Element => Self::Element(ElementDefinition::parse(rest)?),
            DefinitionKind::Entity => Self::Entity(EntityDefinition::parse(rest)?),
            DefinitionKind::LineTag => Self::LineTag(LineTagDefinition::parse(rest)?),
        })
    }
}

impl<'a> From<AttributeListDefinition<'a>> for Definition<'a> {
    fn from(value: AttributeListDefinition<'a>) -> Self {
        Self::AttributeList(value)
    }
}
impl<'a> From<ElementDefinition<'a>> for Definition<'a> {
    fn from(value: ElementDefinition<'a>) -> Self {
        Self::Element(value)
    }
}
impl<'a> From<EntityDefinition<'a>> for Definition<'a> {
    fn from(value: EntityDefinition<'a>) -> Self {
        Self::Entity(value)
    }
}
impl<'a> From<LineTagDefinition<'a>> for Definition<'a> {
    fn from(value: LineTagDefinition<'a>) -> Self {
        Self::LineTag(value)
    }
}
impl<'a> TryFrom<Definition<'a>> for AttributeListDefinition<'a> {
    type Error = TryFromNodeError;

    fn try_from(value: Definition<'a>) -> Result<Self, Self::Error> {
        let got = match value {
            Definition::AttributeList(def) => return Ok(def),
            Definition::Element(_) => "Element",
            Definition::Entity(_) => "Entity",
            Definition::LineTag(_) => "LineTag",
        };
        Err(TryFromNodeError {
            prefix: "Definition",
            expected: "AttributeList",
            got,
        })
    }
}
impl<'a> TryFrom<Definition<'a>> for ElementDefinition<'a> {
    type Error = TryFromNodeError;

    fn try_from(value: Definition<'a>) -> Result<Self, Self::Error> {
        let got = match value {
            Definition::AttributeList(_) => "AttributeList",
            Definition::Element(def) => return Ok(def),
            Definition::Entity(_) => "Entity",
            Definition::LineTag(_) => "LineTag",
        };
        Err(TryFromNodeError {
            prefix: "Definition",
            expected: "Element",
            got,
        })
    }
}
impl<'a> TryFrom<Definition<'a>> for EntityDefinition<'a> {
    type Error = TryFromNodeError;

    fn try_from(value: Definition<'a>) -> Result<Self, Self::Error> {
        let got = match value {
            Definition::AttributeList(_) => "AttributeList",
            Definition::Element(_) => "Element",
            Definition::Entity(def) => return Ok(def),
            Definition::LineTag(_) => "LineTag",
        };
        Err(TryFromNodeError {
            prefix: "Definition",
            expected: "Entity",
            got,
        })
    }
}
impl<'a> TryFrom<Definition<'a>> for LineTagDefinition<'a> {
    type Error = TryFromNodeError;

    fn try_from(value: Definition<'a>) -> Result<Self, Self::Error> {
        let got = match value {
            Definition::AttributeList(_) => "AttributeList",
            Definition::Element(_) => "Element",
            Definition::Entity(_) => "Entity",
            Definition::LineTag(def) => return Ok(def),
        };
        Err(TryFromNodeError {
            prefix: "Definition",
            expected: "LineTag",
            got,
        })
    }
}
macro_rules! impl_try_from_tag {
    ($t:ident) => {
        impl<'a> From<$t<'a>> for super::Tag<'a> {
            fn from(value: $t<'a>) -> Self {
                Self::Definition(value.into())
            }
        }
        impl<'a> TryFrom<super::Tag<'a>> for $t<'a> {
            type Error = TryFromNodeError;

            fn try_from(value: super::Tag<'a>) -> Result<Self, Self::Error> {
                Definition::try_from(value)?.try_into()
            }
        }
    };
}
impl_try_from_tag!(AttributeListDefinition);
impl_try_from_tag!(ElementDefinition);
impl_try_from_tag!(EntityDefinition);
impl_try_from_tag!(LineTagDefinition);
