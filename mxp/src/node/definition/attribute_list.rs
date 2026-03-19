use std::fmt;

use crate::ErrorKind;
use crate::parse::Words;

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
        write!(f, "<!AT {name} '{attributes}'>")
    }
}

impl<'a> AttributeListDefinition<'a> {
    pub(super) fn parse(mut words: Words<'a>) -> crate::Result<Self> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_attlist() {
        let def = AttributeListDefinition {
            name: "custom",
            attributes: "color=red background=white flags",
        };
        assert_eq!(
            def.to_string(),
            "<!AT custom 'color=red background=white flags'>"
        );
    }
}
