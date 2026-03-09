use std::str::FromStr;

use flagset::flags;

use crate::parser::UnrecognizedVariant;

flags! {
    pub(crate) enum ElementKeyword: u8 {
        Open,
        Empty,
        Delete,
    }

    pub enum EntityKeyword: u8 {
        Private,
        Publish,
        Delete,
        Add,
        Remove,
    }

    pub(crate) enum FrameKeyword: u8 {
        Floating,
        Internal,
    }

    pub(crate) enum ImageKeyword: u8 {
        IsMap,
    }

    pub enum MxpKeyword: u8 {
        Off,
        DefaultLocked,
        DefaultSecure,
        DefaultOpen,
        IgnoreNewlines,
        UseNewlines,
    }

    pub(crate) enum SendKeyword: u8 {
        Prompt
    }

    pub(crate) enum TagKeyword: u8 {
        Gag,
        Enable,
        Disable,
    }
}

impl FromStr for ElementKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "open" => Ok(Self::Open),
            "empty" => Ok(Self::Empty),
            "delete" => Ok(Self::Delete),
            _ => Err(Self::Err::new(s)),
        }
    }
}

impl FromStr for EntityKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "private" => Ok(Self::Private),
            "publish" => Ok(Self::Publish),
            "delete" => Ok(Self::Delete),
            "add" => Ok(Self::Add),
            "remove" => Ok(Self::Remove),
            _ => Err(Self::Err::new(s)),
        }
    }
}

impl FromStr for FrameKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "floating" => Ok(Self::Floating),
            "internal" => Ok(Self::Internal),
            _ => Err(Self::Err::new(s))
        }
    }
}

impl FromStr for ImageKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "ismap" => Ok(Self::IsMap),
            _ =>  Err(Self::Err::new(s))
        }
    }
}

impl FromStr for MxpKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "off" => Ok(Self::Off),
            "defaultopen" => Ok(Self::DefaultOpen),
            "defaultsecure" => Ok(Self::DefaultSecure),
            "defaultlocked" => Ok(Self::DefaultLocked),
            "usenewlines" => Ok(Self::UseNewlines),
            "ignorenewlines" => Ok(Self::IgnoreNewlines),
            _ =>  Err(Self::Err::new(s)),
        }
    }
}

impl FromStr for SendKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "prompt" => Ok(Self::Prompt),
            _ => Err(Self::Err::new(s)),
        }
    }
}

impl FromStr for TagKeyword {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "gag" => Ok(Self::Gag),
            "enable" => Ok(Self::Enable),
            "disable" => Ok(Self::Disable),
            _ => Err(Self::Err::new(s))
        }
    }
}
