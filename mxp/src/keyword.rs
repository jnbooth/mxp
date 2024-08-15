use std::str::FromStr;

use enumeration::Enum;

use crate::parser::UnrecognizedVariant;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum ElementKeyword {
    Open,
    Empty,
    Delete,
}

impl FromStr for ElementKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "OPEN" => Self::Open,
            "EMPTY" => Self::Empty,
            "DELETE" => Self::Delete,
            _ => return Err(Self::Err::new(s)),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum EntityKeyword {
    Private,
    Publish,
    Delete,
    Add,
    Remove,
}

impl FromStr for EntityKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "PRIVATE" => Self::Private,
            "PUBLISH" => Self::Publish,
            "DELETE" => Self::Delete,
            "ADD" => Self::Add,
            "REMOVE" => Self::Remove,
            _ => return Err(Self::Err::new(s)),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum FrameKeyword {
    Floating,
    Internal,
}

impl FromStr for FrameKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "FLOATING" => Self::Floating,
            "INTERNAL" => Self::Internal,
            _ => return Err(Self::Err::new(s))
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum ImageKeyword {
    IsMap,
}

impl FromStr for ImageKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "ISMAP" => Self::IsMap,
            _ => return Err(Self::Err::new(s))
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum MxpKeyword {
    Off,
    DefaultLocked,
    DefaultSecure,
    DefaultOpen,
    IgnoreNewlines,
    UseNewlines,
}

impl FromStr for MxpKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "OFF" => Self::Off,
            "DEFAULTOPEN" => Self::DefaultOpen,
            "DEFAULTSECURE" => Self::DefaultSecure,
            "DEFAULTLOCKED" => Self::DefaultLocked,
            "USENEWLINES" => Self::UseNewlines,
            "IGNORENEWLINES" => Self::IgnoreNewlines,
            _ => return Err(Self::Err::new(s)),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum SendKeyword {
    Prompt,
}

impl FromStr for SendKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "PROMPT" => Self::Prompt,
            _ => return Err(Self::Err::new(s))
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum TagKeyword {
    Gag,
    Enable,
    Disable,
}

impl FromStr for TagKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "GAG" => Self::Gag,
            "ENABLE" => Self::Enable,
            "DISABLE" => Self::Disable,
            _ => return Err(Self::Err::new(s))
        })
    }
}
