use std::collections::HashSet;

use flagset::{FlagSet, flags};
use mxp::RgbColor;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum UseMxp {
    /// Activate MXP in response to an `IAC` subnegotiation from the server setting MXP on.
    #[default]
    Command,
    /// Activate MXP in response to a telnet `WILL` query from the server about the MXP protocol.
    Query,
    /// Always activate MXP.
    Always,
    /// Never activate MXP.
    Never,
}

flags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
        Sound,
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
            Tag::Sound => Self::Sound,
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
    /// Transmitted in response to an MXP `<VERSION>` request.
    /// Default: empty.
    pub app_name: String,
    /// Overrides for ANSI/XTerm colors. 0 = black, 1 = maroon, etc.
    /// Default: empty.
    pub colors: Vec<RgbColor>,
    /// Client supports some VT100 features, such as moving the cursor and erasing characters.
    /// Default: false.
    pub console_controls: bool,
    /// Insert a newline into the output when server sends GA (Go-Ahead).
    /// Default: false.
    pub convert_ga_to_newline: bool,
    /// Disable MCCP (Mud Client Compression Protocol).
    /// Default: false.
    pub disable_compression: bool,
    /// Use the US-ASCII charset instead of UTF-8.
    /// Default: false.
    pub disable_utf8: bool,
    /// Ignore colors set by MXP tags.
    /// Default: false.
    pub ignore_mxp_colors: bool,
    /// Client supports XTerm mouse tracking.
    /// Default: false.
    pub mouse_tracking: bool,
    /// Client supports NAWS (Negotiate About Window Size) protocol.
    /// Default: false.
    pub naws: bool,
    /// Ignore server requests to turn local echo off.
    /// Default: false.
    pub no_echo_off: bool,
    /// Transmitted in response to an MXP `<PASSWORD>` request.
    /// Default: empty.
    pub password: String,
    /// Transmitted in response to an MXP `<USER>` request.
    /// Default: empty.
    pub player: String,
    /// Client is a proxy allowing different users to connect from the same IP address.
    /// Default: false.
    pub proxy: bool,
    /// Client is using a screen reader.
    /// Default: false.
    pub screen_reader: bool,
    /// Client supports SSL for data encryption, preferably TLS 1.3 or higher.
    /// Default: false.
    pub ssl: bool,
    /// MXP tags supported by the client.
    /// Default: all tags.
    pub supports: FlagSet<Tag>,
    /// String used to identify the terminal to the server.
    /// Default: empty.
    pub terminal_identification: String,
    /// Condition for the transformer to activate MXP mode.
    /// Default: [`UseMxp::Command`].
    pub use_mxp: UseMxp,
    /// Transmitted in response to an MXP `<VERSION>` response.
    /// Default: empty.
    pub version: String,
    /// Custom Telnet protocols to support, sent during a telnet `WILL` negotiation.
    /// Default: empty.
    pub will: HashSet<u8>,
    /// Client supports some XTerm features, such as setting window titles and minimizing windows.
    /// Default: false.
    pub window_controls: bool,
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
            console_controls: false,
            convert_ga_to_newline: false,
            disable_compression: false,
            disable_utf8: false,
            ignore_mxp_colors: false,
            mouse_tracking: false,
            naws: false,
            no_echo_off: false,
            password: String::new(),
            player: String::new(),
            proxy: false,
            screen_reader: false,
            ssl: false,
            supports: FlagSet::full(),
            terminal_identification: String::new(),
            use_mxp: UseMxp::Command,
            version: String::new(),
            will: HashSet::new(),
            window_controls: false,
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
