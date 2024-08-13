use enumeration::Enum;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum XchMode {
    Text,
    Html,
    PureHtml,
}

impl XchMode {
    pub fn parse(s: &str) -> Option<Self> {
        match_ci! {s,
            "text" => Some(Self::Text),
            "html" => Some(Self::Html),
            "purehtml" => Some(Self::PureHtml),
            _ => None,
        }
    }
}
