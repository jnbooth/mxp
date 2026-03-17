use std::fmt;
use std::str::FromStr;

use flagset::FlagSet;

use super::error::TryFromNodeError;
use crate::arguments::{ArgumentScanner, Arguments, ExpectArg as _};
use crate::color::RgbColor;
use crate::element::{Element, ElementItem};
use crate::keyword::{ElementKeyword, EntityKeyword, LineTagKeyword};
use crate::line::Mode;
use crate::parse::Words;
use crate::{Error, ErrorKind};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum DefinitionKind {
    AttributeList,
    Element,
    Entity,
    LineTag,
}

impl FromStr for DefinitionKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "attlist" | "att" => Ok(Self::AttributeList),
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
    /// `<!ATTLIST ...>` or `<!ATT ...>`.
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
        let mut words = Words::new(source);
        let kind = words
            .next()
            .ok_or_else(|| Error::new("<!>", ErrorKind::IncompleteElement))?
            .parse()?;
        Ok(match kind {
            DefinitionKind::AttributeList => {
                Self::AttributeList(AttributeListDefinition::parse(words)?)
            }
            DefinitionKind::Element => Self::Element(ElementDefinition::parse(words)?),
            DefinitionKind::Entity => Self::Entity(EntityDefinition::parse(words)?),
            DefinitionKind::LineTag => Self::LineTag(LineTagDefinition::parse(words)?),
        })
    }
}

/// Syntax tree of an attribute list definition from the server, in the form of
/// `<!ATTLIST {name} ...>`.
///
/// Full definition:
///
/// ```xml
/// <!ATTLIST
///     Name
///     Attributes
/// >
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AttributeListDefinition<'a> {
    /// Name of the element for which the additional attributes are being defined.
    pub name: &'a str,
    /// [`State::define`](crate::State::define) forwards the attributes directly to the previously
    /// defined arguments.
    pub attributes: &'a str,
}

impl fmt::Display for AttributeListDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { name, attributes } = self;
        write!(f, "<!ATT {name} {attributes}>")
    }
}

impl<'a> AttributeListDefinition<'a> {
    fn parse(mut words: Words<'a>) -> crate::Result<Self> {
        let name = words.next_or(ErrorKind::IncompleteElement)?;
        crate::validate(name, ErrorKind::InvalidElementName)?;
        let attributes = words.as_str();
        let attributes = Self::unquote(attributes).unwrap_or(attributes);
        Ok(Self { name, attributes })
    }

    fn unquote(s: &str) -> Option<&str> {
        let s = s.trim().strip_prefix('\'')?.strip_suffix('\'')?;
        if s.contains('\'') { None } else { Some(s) }
    }
}

/// Syntax tree of an entity definition from the server, in the form of `<!ENTITY {name} ...>`.
///
/// Full definition:
///
/// ```xml
/// <!ELEMENT
///     Name
///     [Definition]
///     [ATT=attribute-list]
///     [TAG=tag]
///     [FLAG=flags]
///     [OPEN]
///     [DELETE]
///     [EMPTY]
/// >
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementDefinition<'a> {
    /// Name of the element.
    pub name: &'a str,
    /// Definition of the element, or `None` if this is a `DELETE` instruction.
    pub element: Option<Element>,
}

impl fmt::Display for ElementDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { name, element } = self;
        if let Some(element) = element {
            element.fmt(f)
        } else {
            write!(f, "<!EL {name} DELETE>")
        }
    }
}

impl<'a> ElementDefinition<'a> {
    fn parse(mut words: Words<'a>) -> crate::Result<Self> {
        let name = words.next_or(ErrorKind::IncompleteElement)?;
        crate::validate(name, ErrorKind::InvalidElementName)?;
        let args = words.parse_args()?;

        let mut iter = args.scan(()).with_keywords();

        let items = match iter.get_next() {
            Some(&arg) => ElementItem::parse_all(arg)?,
            None => Vec::new(),
        };

        let attributes = match iter.get_named("att") {
            Some(&atts) => Words::new(atts).try_into()?,
            None => Arguments::default(),
        };

        let tag = match iter.get_named("tag") {
            Some(&tag) => match tag.parse() {
                Ok(tag) if Mode(tag).is_user_defined() => Some(Mode(tag)),
                _ => {
                    return Err(crate::Error::new(
                        tag,
                        ErrorKind::IllegalLineTagInDefinition,
                    ));
                }
            },
            None => None,
        };

        let (parse_as, variable) = match iter.get_named("flag") {
            Some(&flag) if flag[.."set ".len()].eq_ignore_ascii_case("set ") => {
                (None, Some(flag["set ".len()..].to_owned()))
            }
            Some(&flag) => (Some(flag.parse()?), None),
            None => (None, None),
        };

        let keywords = iter.into_keywords()?;

        if keywords.contains(ElementKeyword::Delete) {
            return Ok(Self {
                name,
                element: None,
            });
        }

        Ok(Self {
            name,
            element: Some(Element {
                name: name.to_owned(),
                open: keywords.contains(ElementKeyword::Open),
                command: keywords.contains(ElementKeyword::Empty),
                items,
                attributes,
                line_tag: tag,
                parse_as,
                variable,
            }),
        })
    }
}

/// Syntax tree of an entity definition from the server, in the form of
/// `<!ENTITY {name} {value} ...>`.
///
/// Full definition:
///
/// ```xml
/// <!ENTITY
///     Name
///     Value
///     [DESC=description]
///     [PRIVATE]
///     [PUBLISH]
///     [DELETE]
///     [ADD]
///     [REMOVE]
/// >
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EntityDefinition<'a> {
    /// Name of the entity.
    pub name: &'a str,
    /// Value of the entity.
    pub value: &'a str,
    /// Optional description of the entity.
    pub desc: Option<&'a str>,
    /// Set of keywords included in the definition.
    pub keywords: FlagSet<EntityKeyword>,
}

impl fmt::Display for EntityDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Self {
            name,
            value,
            desc,
            keywords,
        } = self;
        write!(f, "<!EN {name} \"{value}\"")?;
        if let Some(desc) = desc {
            write!(f, " DESC=\"{desc}\"")?;
        }
        for keyword in keywords {
            write!(f, " {keyword}")?;
        }
        f.write_str(">")
    }
}

impl<'a> EntityDefinition<'a> {
    fn parse(mut words: Words<'a>) -> crate::Result<Self> {
        let source = words.source();
        let name = words.next_or(ErrorKind::IncompleteElement)?;
        crate::validate(name, ErrorKind::InvalidElementName)?;
        let args = words.parse_args()?;
        let mut scanner = args.scan(()).with_keywords();
        let Some(value) = scanner.get_next() else {
            return Err(Error::new(source, ErrorKind::EmptyElementInDefinition));
        };
        let desc = scanner.get_named("desc").copied();
        let keywords = scanner.into_keywords()?;
        Ok(Self {
            name,
            value,
            desc,
            keywords,
        })
    }
}

/// Parsed representation of a line tag definition from the server, in the form of
/// `<!TAG {index} ...>`.
///
/// Full definition:
///
/// ```xml
/// <!TAG
///     Index
///     [WINDOW=string]
///     [FORE=color]
///     [BACK=color]
///     [GAG]
///     [ENABLE]
///     [DISABLE]
/// >
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LineTagDefinition<'a> {
    /// Tag number (20-99) to change.
    pub index: Mode,
    /// Window to redirect the text to.
    pub window: Option<&'a str>,
    /// Text should be gagged from the main MUD window.
    pub gag: Option<bool>,
    /// Text color.
    pub fore: Option<RgbColor>,
    /// Background color of the text.
    pub back: Option<RgbColor>,
    /// If `Some(true)`, activates the line tag. If `Some(false)`, deactivates it.
    pub enable: Option<bool>,
}

impl fmt::Display for LineTagDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Self {
            index,
            window,
            gag,
            fore,
            back,
            enable,
        } = self;
        write!(f, "<!TAG {index}")?;
        if let Some(window) = window {
            write!(f, " WINDOWNAME=\"{window}\"")?;
        }
        if gag == Some(true) {
            f.write_str(" GAG")?;
        }
        if let Some(fore) = fore {
            write!(f, " FORE={fore}")?;
        }
        if let Some(back) = back {
            write!(f, " BACK={back}")?;
        }
        match enable {
            Some(true) => f.write_str(" ENABLE"),
            Some(false) => f.write_str(" DISABLE"),
            None => Ok(()),
        }
    }
}

impl<'a> LineTagDefinition<'a> {
    fn parse(words: Words<'a>) -> crate::Result<Self> {
        let args = words.parse_args()?;
        let mut scanner = args.scan(()).with_keywords();
        let index = Mode(scanner.get_next().expect_number()?.expect_some("Tag")?);
        if !index.is_user_defined() {
            return Err(Error::new(index.to_string(), ErrorKind::IllegalLineTag));
        }
        let window = scanner.get_named("windowname").copied();
        let fore = scanner.get_named("fore").expect_color()?;
        let back = scanner.get_named("back").expect_color()?;
        let keywords = scanner.into_keywords()?;
        let gag = if keywords.contains(LineTagKeyword::Gag) {
            Some(true)
        } else {
            None
        };
        let enable = if keywords.contains(LineTagKeyword::Disable) {
            Some(false)
        } else if keywords.contains(LineTagKeyword::Enable) {
            Some(true)
        } else {
            None
        };
        Ok(Self {
            index,
            window,
            gag,
            fore,
            back,
            enable,
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
