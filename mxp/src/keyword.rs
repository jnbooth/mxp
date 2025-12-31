use std::str::FromStr;

use flagset::flags;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::parser::UnrecognizedVariant;

flags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialOrd, Ord, Hash)]
    pub enum ElementKeyword: u8 {
        Open,
        Empty,
        Delete,
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialOrd, Ord, Hash)]
    pub enum EntityKeyword: u8 {
        Private,
        Publish,
        Delete,
        Add,
        Remove,
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialOrd, Ord, Hash)]
    pub enum FrameKeyword: u8 {
        Floating,
        Internal,
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialOrd, Ord, Hash)]
    pub enum ImageKeyword: u8 {
        IsMap,
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialOrd, Ord, Hash)]
    pub enum MxpKeyword: u8 {
        Off,
        DefaultLocked,
        DefaultSecure,
        DefaultOpen,
        IgnoreNewlines,
        UseNewlines,
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialOrd, Ord, Hash)]
    pub enum SendKeyword: u8 {
        Prompt
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialOrd, Ord, Hash)]
    pub enum TagKeyword: u8 {
        Gag,
        Enable,
        Disable,
    }
}

impl FromStr for ElementKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "open" => Self::Open,
            "empty" => Self::Empty,
            "delete" => Self::Delete,
            _ => return Err(Self::Err::new(s)),
        })
    }
}

impl FromStr for EntityKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "private" => Self::Private,
            "publish" => Self::Publish,
            "delete" => Self::Delete,
            "add" => Self::Add,
            "remove" => Self::Remove,
            _ => return Err(Self::Err::new(s)),
        })
    }
}

impl FromStr for FrameKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "floating" => Self::Floating,
            "internal" => Self::Internal,
            _ => return Err(Self::Err::new(s))
        })
    }
}

impl FromStr for ImageKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "ismap" => Self::IsMap,
            _ => return Err(Self::Err::new(s))
        })
    }
}

impl FromStr for MxpKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "off" => Self::Off,
            "defaultopen" => Self::DefaultOpen,
            "defaultsecure" => Self::DefaultSecure,
            "defaultlocked" => Self::DefaultLocked,
            "usenewlines" => Self::UseNewlines,
            "ignorenewlines" => Self::IgnoreNewlines,
            _ => return Err(Self::Err::new(s)),
        })
    }
}

impl FromStr for SendKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "prompt" => Self::Prompt,
            _ => return Err(Self::Err::new(s))
        })
    }
}

impl FromStr for TagKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "gag" => Self::Gag,
            "enable" => Self::Enable,
            "disable" => Self::Disable,
            _ => return Err(Self::Err::new(s))
        })
    }
}
