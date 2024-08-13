use std::str::FromStr;

use enumeration::Enum;

use crate::parser::UnrecognizedVariant;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum XchMode {
    Text,
    Html,
    PureHtml,
}

impl FromStr for XchMode {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "text" => Self::Text,
            "html" => Self::Html,
            "purehtml" => Self::PureHtml,
            _ => return Err(Self::Err::new(s)),
        })
    }
}
