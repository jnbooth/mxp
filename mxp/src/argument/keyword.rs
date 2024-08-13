use std::str::FromStr;

use enumeration::Enum;

use crate::parser::UnrecognizedVariant;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Keyword {
    Delete,
    Open,
    Empty,
    Prompt,
    Off,
    DefaultOpen,
    DefaultSecure,
    DefaultLocked,
    UseNewlines,
    IgnoreNewlines,
    IsMap,
}

impl FromStr for Keyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "delete" => Self::Delete,
            "open" => Self::Open,
            "empty" => Self::Empty,
            "prompt" => Self::Prompt,
            "off" => Self::Off,
            "defaultopen" => Self::DefaultOpen,
            "defaultsecure" => Self::DefaultSecure,
            "defaultlocked" => Self::DefaultLocked,
            "usenewlines" => Self::UseNewlines,
            "ignorenewlines" => Self::IgnoreNewlines,
            "ismap" => Self::IsMap,
            _ => return Err(Self::Err::new(s)),
        })
    }
}
