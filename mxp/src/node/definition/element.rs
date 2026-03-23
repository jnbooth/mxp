use std::fmt;

use crate::arguments::{ArgumentScanner, Arguments};
use crate::element::{AttributeList, Element, ElementFlag, ElementItem};
use crate::keyword::ElementKeyword;
use crate::line::Mode;
use crate::parse::{ArgumentParser, split_name, validate};
use crate::{Error, ErrorKind};

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
    pub(super) fn parse(source: &'a str) -> crate::Result<Self> {
        let (name, args) = split_name(source);
        if name.is_empty() {
            return Err(Error::new(
                "empty element definition",
                ErrorKind::IncompleteElement,
            ));
        }
        validate(name, ErrorKind::InvalidElementName)?;

        let args = Arguments::parse(args)?;
        let mut iter = args.scan().with_keywords();

        let items = match iter.get_next()? {
            Some(&arg) => ElementItem::parse_all(arg)?,
            None => Vec::new(),
        };

        let attributes = match iter.get_named("att")? {
            Some(&atts) => ArgumentParser::new(atts).try_into()?,
            None => AttributeList::default(),
        };

        let tag = match iter.get_named("tag")? {
            Some(&tag) => match tag.parse() {
                Ok(tag) if Mode(tag).is_user_defined() => Some(Mode(tag)),
                _ => {
                    return Err(Error::new(tag, ErrorKind::IllegalLineTagInDefinition));
                }
            },
            None => None,
        };

        let flag = match iter.get_named("flag")? {
            Some(&flag) => Some(ElementFlag::parse(flag)?),
            None => None,
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
                empty: keywords.contains(ElementKeyword::Empty),
                items,
                attributes,
                line_tag: tag,
                flag,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_element_off() {
        let def = ElementDefinition {
            name: "custom",
            element: None,
        };
        assert_eq!(def.to_string(), "<!EL custom DELETE>");
    }
}
