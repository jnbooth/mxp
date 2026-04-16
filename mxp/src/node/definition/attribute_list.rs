use std::fmt;

use crate::parse::{split_name, validate};
use crate::{Error, ErrorKind};

/// Syntax tree of an attribute list definition from the server, in the form of
/// `<!ATTLIST {name} ...>`.
///
/// See [`MXP specification: Attributes`](https://www.zuggsoft.com/zmud/mxp.htm#ATTLIST).
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
    pub(super) fn parse(source: &'a str) -> crate::Result<Self> {
        let (name, attributes) = split_name(source);
        if name.is_empty() {
            return Err(Error::new(
                "empty attribute list",
                ErrorKind::IncompleteElement,
            ));
        }
        validate(name, ErrorKind::InvalidElementName)?;
        Ok(Self {
            name,
            attributes: Self::unquote(attributes),
        })
    }

    fn unquote(s: &str) -> &str {
        let s = match s.as_bytes() {
            [b'\'', s @ .., b'\''] | [b'\'', s @ ..] | [s @ .., b'\''] | s => s,
        };
        // SAFETY: Valid UTF-8.
        unsafe { str::from_utf8_unchecked(s) }
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
