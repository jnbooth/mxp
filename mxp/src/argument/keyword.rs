use enumeration::Enum;

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

impl Keyword {
    pub fn parse(s: &str) -> Option<Self> {
        match_ci! {s,
            "delete" => Some(Self::Delete),
            "open" => Some(Self::Open),
            "empty" => Some(Self::Empty),
            "prompt" => Some(Self::Prompt),
            "off" => Some(Self::Off),
            "defaultopen" => Some(Self::DefaultOpen),
            "defaultsecure" => Some(Self::DefaultSecure),
            "defaultlocked" => Some(Self::DefaultLocked),
            "usenewlines" => Some(Self::UseNewlines),
            "ignorenewlines" => Some(Self::IgnoreNewlines),
            "ismap" => Some(Self::IsMap),
            _ => None,
        }
    }
}
