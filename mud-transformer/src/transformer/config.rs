use std::collections::HashSet;

use flagset::{FlagSet, flags};
use mxp::RgbColor;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

flags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialOrd, Ord, Hash)]
    pub enum UseMxp: u8 {
        Command,
        Query,
        Always,
        Never,
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialOrd, Ord, Hash)]
    pub enum Tag: u32 {
        Bold,
        Color,
        Dest,
        Expire,
        Filter,
        Font,
        Frame,
        Gauge,
        H1,
        H2,
        H3,
        H4,
        H5,
        H6,
        Highlight,
        Hr,
        Hyperlink,
        Image,
        Italic,
        Music,
        Relocate,
        Send,
        Small,
        Stat,
        Strikeout,
        Tt,
        Underline,
    }
}

impl From<Tag> for mxp::ActionKind {
    fn from(value: Tag) -> Self {
        match value {
            Tag::Bold => Self::Bold,
            Tag::Color => Self::Color,
            Tag::Dest => Self::Dest,
            Tag::Expire => Self::Expire,
            Tag::Filter => Self::Filter,
            Tag::Font => Self::Font,
            Tag::Frame => Self::Frame,
            Tag::Gauge => Self::Gauge,
            Tag::H1 => Self::H1,
            Tag::H2 => Self::H2,
            Tag::H3 => Self::H3,
            Tag::H4 => Self::H4,
            Tag::H5 => Self::H5,
            Tag::H6 => Self::H6,
            Tag::Highlight => Self::Highlight,
            Tag::Hr => Self::Hr,
            Tag::Hyperlink => Self::Hyperlink,
            Tag::Image => Self::Image,
            Tag::Italic => Self::Italic,
            Tag::Music => Self::Music,
            Tag::Relocate => Self::Relocate,
            Tag::Send => Self::Send,
            Tag::Small => Self::Small,
            Tag::Stat => Self::Stat,
            Tag::Strikeout => Self::Strikeout,
            Tag::Tt => Self::Tt,
            Tag::Underline => Self::Underline,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransformerConfig {
    pub app_name: String,
    pub colors: Vec<RgbColor>,
    pub convert_ga_to_newline: bool,
    pub disable_compression: bool,
    pub disable_utf8: bool,
    pub ignore_mxp_colors: bool,
    pub naws: bool,
    pub no_echo_off: bool,
    pub password: String,
    pub player: String,
    pub screen_reader: bool,
    pub ssl: bool,
    pub supports: FlagSet<Tag>,
    pub terminal_identification: String,
    pub use_mxp: UseMxp,
    pub version: String,
    pub will: HashSet<u8>,
}

impl Default for TransformerConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformerConfig {
    pub fn new() -> Self {
        Self {
            app_name: String::new(),
            colors: Vec::new(),
            convert_ga_to_newline: false,
            disable_compression: false,
            disable_utf8: false,
            ignore_mxp_colors: false,
            naws: false,
            no_echo_off: false,
            password: String::new(),
            player: String::new(),
            screen_reader: false,
            ssl: false,
            supports: FlagSet::full(),
            terminal_identification: String::new(),
            use_mxp: UseMxp::Command,
            version: String::new(),
            will: HashSet::new(),
        }
    }

    pub(crate) fn supported_actions(&self) -> FlagSet<mxp::ActionKind> {
        let mut actions = FlagSet::full();
        for action in (!self.supports).into_iter().map(mxp::ActionKind::from) {
            actions -= action;
        }
        actions
    }
}
