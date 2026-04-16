use std::fmt;

use super::parse_as::ParseAs;
use crate::parse::UnrecognizedVariant;

#[derive(Clone, Debug, PartialEq, Eq)]
/// The `FLAG` argument in an [`Element`] definition, which assigns an internal action to the
/// element.
///
/// See [MXP specification: Tag Properties](https://www.zuggsoft.com/zmud/mxp.htm#Tag%20Properties).
///
/// [`Element`]: crate::Element
pub enum ElementFlag {
    /// If specified, text contained by this element should be parsed in a specific way by an
    /// automapper.
    ParseAs(ParseAs),
    /// If specified, text contained by this element should be stored as a local variable with the
    /// given name.
    Set(String),
}

impl fmt::Display for ElementFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParseAs(parse_as) => parse_as.fmt(f),
            Self::Set(variable) => write!(f, "SET {variable}"),
        }
    }
}

impl ElementFlag {
    pub(crate) fn parse(s: &str) -> Result<Self, UnrecognizedVariant<ParseAs>> {
        match s.split_at_checked(4) {
            Some((a, b)) if a.eq_ignore_ascii_case("set ") => Ok(Self::Set(b.to_owned())),
            _ => s.parse().map(Self::ParseAs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_as() {
        assert_eq!(
            ElementFlag::parse("ROOMNUM").unwrap(),
            ElementFlag::ParseAs(ParseAs::RoomNum)
        );
    }

    #[test]
    fn set() {
        assert_eq!(
            ElementFlag::parse("SET test").unwrap(),
            ElementFlag::Set("test".to_owned())
        );
    }

    #[test]
    fn invalid() {
        assert!(ElementFlag::parse("RESET test").is_err());
    }
}
