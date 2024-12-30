use flagset::flags;
use std::str::FromStr;

use crate::parser::UnrecognizedVariant;

flags! {
    #[derive(PartialOrd, Ord, Hash)]
    pub enum ElementKeyword: u8 {
        Open,
        Empty,
        Delete,
    }

    #[derive(PartialOrd, Ord, Hash)]
    pub enum EntityKeyword: u8 {
        Private,
        Publish,
        Delete,
        Add,
        Remove,
    }

    #[derive(PartialOrd, Ord, Hash)]
    pub enum FrameKeyword: u8 {
        Floating,
        Internal,
    }

    #[derive(PartialOrd, Ord, Hash)]
    pub enum ImageKeyword: u8 {
        IsMap,
    }

    #[derive(PartialOrd, Ord, Hash)]
    pub enum MxpKeyword: u8 {
        Off,
        DefaultLocked,
        DefaultSecure,
        DefaultOpen,
        IgnoreNewlines,
        UseNewlines,
    }

    #[derive(PartialOrd, Ord, Hash)]
    pub enum SendKeyword: u8 {
        Prompt
    }

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
            "OPEN" => Self::Open,
            "EMPTY" => Self::Empty,
            "DELETE" => Self::Delete,
            _ => return Err(Self::Err::new(s)),
        })
    }
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

impl FromStr for ImageKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "ISMAP" => Self::IsMap,
            _ => return Err(Self::Err::new(s))
        })
    }
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

impl FromStr for SendKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "PROMPT" => Self::Prompt,
            _ => return Err(Self::Err::new(s))
        })
    }
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
