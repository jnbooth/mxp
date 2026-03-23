use std::fmt;

use flagset::FlagSet;

use crate::arguments::{ArgumentScanner, Arguments};
use crate::keyword::EntityKeyword;
use crate::parse::{split_name, validate};
use crate::{Error, ErrorKind};

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
    pub(super) fn parse(source: &'a str) -> crate::Result<Self> {
        let (name, args) = split_name(source);
        if name.is_empty() {
            return Err(Error::new(
                "empty entity definition",
                ErrorKind::IncompleteElement,
            ));
        }
        validate(name, ErrorKind::InvalidEntityName)?;
        let args = Arguments::parse(args)?;
        let mut scanner = args.scan().with_keywords();
        let Some(value) = scanner.get_next()? else {
            return Err(Error::new(source, ErrorKind::EmptyElementInDefinition));
        };
        let desc = scanner.get_named("desc")?.copied();
        let keywords = scanner.into_keywords()?;
        Ok(Self {
            name,
            value,
            desc,
            keywords,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_entity() {
        let def = EntityDefinition {
            name: "custom",
            value: "some&nbsp;value",
            desc: Some("mydesc"),
            keywords: EntityKeyword::Publish | EntityKeyword::Add,
        };
        assert_eq!(
            def.to_string(),
            "<!EN custom \"some&nbsp;value\" DESC=\"mydesc\" PUBLISH ADD>"
        );
    }
}
