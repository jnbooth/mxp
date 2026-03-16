use std::str::FromStr;

use flagset::FlagSet;

use super::arguments_str::ArgumentsStr;
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

/// Parsed representation of a definition tag from the server, in the form of `<!...>`.
///
/// Note: This is the parameter type of [`State::define`](crate::State::define).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParsedDefinition<'a> {
    /// `<!ATTLIST ...>` or `<!ATT ...>`.
    AttributeList(AttributeListDefinition<'a>),
    /// `<!ELEMENT ...>` or `<!EL ...>`.
    Element(ElementDefinition<'a>),
    /// `<!ENTITY ...>` or `<!EN ...>`.
    Entity(EntityDefinition<'a>),
    /// `<!TAG ...>`.
    LineTag(LineTagDefinition<'a>),
}

impl<'a> ParsedDefinition<'a> {
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

/// Parsed representation of an attribute list definition from the server, in the form of
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
    /// The rest of the definition as a string slice. This can be parsed to [`Arguments`], but there
    /// is no need to do so; [`State::define`](crate::State::define) forwards the body directly to
    /// the previously defined arguments.
    pub attributes: ArgumentsStr<'a>,
}

impl<'a> AttributeListDefinition<'a> {
    fn parse(mut words: Words<'a>) -> crate::Result<Self> {
        let name = words.next_or(ErrorKind::IncompleteElement)?;
        crate::validate(name, ErrorKind::InvalidElementName)?;
        Ok(Self {
            name,
            attributes: ArgumentsStr(words.as_str()),
        })
    }
}

/// Parsed representation of an entity definition from the server, in the form of
/// `<!ENTITY {name} ...>`.
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

        let attributes = match iter.get_next_or("att") {
            Some(&atts) => Words::new(atts).try_into()?,
            None => Arguments::default(),
        };

        let tag = match iter.get_next_or("tag") {
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

        let (parse_as, variable) = match iter.get_next_or("flag") {
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

/// Parsed representation of an entity definition from the server, in the form of
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
    /// Optional description of the entity.
    pub desc: Option<&'a str>,
    /// Value of the entity.
    pub value: &'a str,
    /// Set of keywords included in the definition.
    pub keywords: FlagSet<EntityKeyword>,
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
        let desc = scanner.get_next_or("desc").copied();
        let keywords = scanner.into_keywords()?;
        Ok(Self {
            name,
            desc,
            value,
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

impl<'a> LineTagDefinition<'a> {
    fn parse(words: Words<'a>) -> crate::Result<Self> {
        let args = words.parse_args()?;
        let mut scanner = args.scan(()).with_keywords();
        let index = Mode(scanner.get_next().expect_number()?.expect_some("Tag")?);
        if !index.is_user_defined() {
            return Err(Error::new(index.to_string(), ErrorKind::IllegalLineTag));
        }
        let window = scanner.get_next_or("windowname").copied();
        let fore = scanner.get_next_or("fore").expect_color()?;
        let back = scanner.get_next_or("back").expect_color()?;
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
