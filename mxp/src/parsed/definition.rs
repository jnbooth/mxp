use std::str::FromStr;

use flagset::FlagSet;

use crate::arguments::{Arguments, ExpectArg as _};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParsedDefinition<'a> {
    AttributeList(AttributeListDefinition<'a>),
    Element(ElementDefinition<'a>),
    Entity(EntityDefinition<'a>),
    LineTag(LineTagDefinition<'a>),
}

impl<'a> ParsedDefinition<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            Self::AttributeList(def) => def.name,
            Self::Element(def) => def.name,
            Self::Entity(def) => def.name,
            Self::LineTag(def) => def.window.unwrap_or_default(),
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AttributeListDefinition<'a> {
    pub name: &'a str,
    pub body: &'a str,
}

impl<'a> AttributeListDefinition<'a> {
    fn parse(mut words: Words<'a>) -> crate::Result<Self> {
        let name = words.next_or(ErrorKind::IncompleteElement)?;
        crate::validate(name, ErrorKind::InvalidElementName)?;
        Ok(Self {
            name,
            body: words.as_str(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementDefinition<'a> {
    pub name: &'a str,
    pub element: Option<Element>,
}

impl<'a> ElementDefinition<'a> {
    fn parse(mut words: Words<'a>) -> crate::Result<Self> {
        let name = words.next_or(ErrorKind::IncompleteElement)?;
        crate::validate(name, ErrorKind::InvalidElementName)?;
        let args = words.parse_args()?;

        let mut iter = args.matcher().with_keywords();

        let items = match iter.next() {
            Some(&arg) => ElementItem::parse_all(arg)?,
            None => Vec::new(),
        };

        let attributes = match iter.next_or("att") {
            Some(&atts) => Words::new(atts).try_into()?,
            None => Arguments::default(),
        };

        let tag = match iter.next_or("tag") {
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

        let (parse_as, variable) = match iter.next_or("flag") {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EntityDefinition<'a> {
    pub name: &'a str,
    pub desc: Option<&'a str>,
    pub value: &'a str,
    pub keywords: FlagSet<EntityKeyword>,
}

impl<'a> EntityDefinition<'a> {
    fn parse(mut words: Words<'a>) -> crate::Result<Self> {
        let source = words.source();
        let name = words.next_or(ErrorKind::IncompleteElement)?;
        crate::validate(name, ErrorKind::InvalidElementName)?;
        let args = words.parse_args()?;
        let mut matcher = args.matcher().with_keywords();
        let Some(value) = matcher.next() else {
            return Err(Error::new(source, ErrorKind::EmptyElementInDefinition));
        };
        let desc = matcher.next_or("desc").copied();
        let keywords = matcher.into_keywords()?;
        Ok(Self {
            name,
            desc,
            value,
            keywords,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LineTagDefinition<'a> {
    pub index: Mode,
    pub window: Option<&'a str>,
    pub gag: Option<bool>,
    pub fore: Option<RgbColor>,
    pub back: Option<RgbColor>,
    pub enable: Option<bool>,
}

impl<'a> LineTagDefinition<'a> {
    fn parse(words: Words<'a>) -> crate::Result<Self> {
        let args = words.parse_args()?;
        let mut matcher = args.matcher().with_keywords();
        let index = Mode(matcher.next().expect_number()?.expect_some("Tag")?);
        if !index.is_user_defined() {
            return Err(Error::new(index.to_string(), ErrorKind::IllegalLineTag));
        }
        let window = matcher.next_or("windowname").copied();
        let fore = matcher.next_or("fore").expect_color()?;
        let back = matcher.next_or("back").expect_color()?;
        let keywords = matcher.into_keywords()?;
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
