use super::font::Font;
use super::frame::{DestArgs, Frame};
use super::image::Image;
use super::link::{ExpireArgs, HyperlinkArgs, Link, SendArgs};
use super::sound::{Music, Sound};
use crate::argument::scan::{AfkArgs, ColorArgs, Decoder, MxpArgs, Scan, SupportArgs, VarArgs};
use crate::color::RgbColor;
use crate::keyword::{EntityKeyword, MxpKeyword};
use enumeration::{Enum, EnumSet};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum ActionType {
    /// eg. <send href="go west"> west
    Send,
    /// bold
    Bold,
    /// underline
    Underline,
    /// italic
    Italic,
    /// eg. <color fore=red back=blue>
    Color,
    /// version request
    Version,
    /// Font appearance
    Font,
    /// play sound
    Sound,
    /// play music
    Music,
    /// send username
    User,
    /// send password
    Password,
    /// causes a new connect to open
    Relocate,
    /// frame
    Frame,
    /// destination frame
    Dest,
    /// show image
    Image,
    /// sound/image filter
    Filter,
    /// Hyperlink (secure)
    Hyperlink,
    /// Hard Line break (secure)
    Br,
    /// Level 1 heading (secure)
    H1,
    /// Level 2 heading (secure)
    H2,
    /// Level 3 heading (secure)
    H3,
    /// Level 4 heading (secure)
    H4,
    /// Level 5 heading (secure)
    H5,
    /// Level 6 heading (secure)
    H6,
    /// Horizontal rule (secure)
    Hr,
    /// ignore next newline
    NoBr,
    /// Soft line break
    SBr,
    /// Paragraph break (secure)
    P,
    /// Strikethrough
    Strikeout,
    /// Client script (secure)
    Script,
    /// Small text
    Small,
    /// Non-proportional font
    Tt,
    /// Unordered list
    Ul,
    /// Ordered list
    Ol,
    /// List item
    Li,
    /// Sample text
    Samp,
    /// Centre text
    Center,
    /// Highlight text
    High,
    /// Set variable
    Var,
    /// AFK - away from keyboard time
    Afk,

    // recent
    /// gauge
    Gauge,
    /// status
    Stat,
    /// expire
    Expire,

    /// close all open tags
    Reset,
    /// MXP command (eg. MXP OFF)
    Mxp,
    /// what commands we support
    Support,

    /// client options set
    SetOption,
    /// server sets option
    RecommendOption,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Heading {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action<S> {
    /// bold
    Bold,
    /// underline
    Underline,
    /// italic
    Italic,
    /// eg. <color fore=red back=blue>
    Color {
        fore: Option<RgbColor>,
        back: Option<RgbColor>,
    },
    /// version request
    Version,
    /// font appearance
    Font(Font<S>),
    /// play sound
    Sound(Sound<S>),
    /// stop all sound
    SoundOff,
    // play music
    Music(Music<S>),
    /// stop all music
    MusicOff,
    /// send username
    User,
    /// send password
    Password,
    /// causes a new connect to open
    Relocate,
    /// frame
    Frame(Frame<S>),
    /// destination frame
    Dest {
        name: S,
    },
    /// show image
    Image(Image<S>),
    /// sound/image filter
    Filter,
    /// Hyperlink or send prompt (secure)
    Link(Link),
    /// Hard Line break (secure)
    Br,
    /// heading (secure)
    Heading(Heading),
    /// Horizontal rule (secure)
    Hr,
    /// ignore next newline
    NoBr,
    /// Soft linebreak
    SBr,
    /// Paragraph break (secure)
    P,
    /// Strikethrough
    Strikeout,
    /// Client script (secure)
    Script,
    /// Small text
    Small,
    /// Non-proportional font
    Tt,
    /// Unordered list
    Ul,
    /// Ordered list
    Ol,
    /// List item
    Li,
    /// Sample text
    Samp,
    /// Centre text
    Center,
    /// Highlight text
    High,
    /// Set variable
    Var {
        variable: S,
        keywords: EnumSet<EntityKeyword>,
    },
    /// AFK - away from keyboard time
    Afk {
        challenge: Option<S>,
    },

    // recent
    /// gauge
    Gauge,
    /// status
    Stat,
    /// expire
    Expire {
        name: Option<S>,
    },

    /// close all open tags
    Reset,
    /// MXP command (eg. MXP OFF)
    Mxp {
        keywords: EnumSet<MxpKeyword>,
    },
    /// what commands we support
    Support {
        supported: Vec<u8>,
    },

    /// client options set
    SetOption,
    /// server sets option
    RecommendOption,
}

impl<S: AsRef<str>> Action<S> {
    pub fn new<'a, D: Decoder>(action: ActionType, scanner: Scan<'a, D>) -> crate::Result<Self>
    where
        D: Decoder<Output<'a> = S>,
    {
        Ok(match action {
            ActionType::Afk => {
                let AfkArgs { challenge } = scanner.try_into()?;
                Self::Afk { challenge }
            }
            ActionType::Bold => Self::Bold,
            ActionType::Br => Self::Br,
            ActionType::Center => Self::Center,
            ActionType::Color => {
                let ColorArgs { fore, back } = scanner.try_into()?;
                Self::Color { fore, back }
            }
            ActionType::Dest => {
                let DestArgs { name } = scanner.try_into()?;
                Self::Dest { name }
            }
            ActionType::Expire => {
                let ExpireArgs { name } = scanner.try_into()?;
                Self::Expire { name }
            }
            ActionType::Filter => Self::Filter,
            ActionType::Font => Self::Font(scanner.try_into()?),
            ActionType::Frame => Self::Frame(scanner.try_into()?),
            ActionType::Gauge => Self::Gauge,
            ActionType::H1 => Self::Heading(Heading::H1),
            ActionType::H2 => Self::Heading(Heading::H2),
            ActionType::H3 => Self::Heading(Heading::H3),
            ActionType::H4 => Self::Heading(Heading::H4),
            ActionType::H5 => Self::Heading(Heading::H5),
            ActionType::H6 => Self::Heading(Heading::H6),
            ActionType::High => Self::High,
            ActionType::Hr => Self::Hr,
            ActionType::Hyperlink => Self::Link(HyperlinkArgs::try_from(scanner)?.into()),
            ActionType::Image => Self::Image(Image::try_from(scanner)?),
            ActionType::Italic => Self::Italic,
            ActionType::Li => Self::Li,
            ActionType::Mxp => {
                let MxpArgs { keywords } = scanner.try_into()?;
                Self::Mxp { keywords }
            }
            ActionType::Music => {
                let music = Music::try_from(scanner)?;
                if music.is_off() {
                    Self::MusicOff
                } else {
                    Self::Music(music)
                }
            }
            ActionType::NoBr => Self::NoBr,
            ActionType::Ol => Self::Ol,
            ActionType::P => Self::P,
            ActionType::Password => Self::Password,
            ActionType::RecommendOption => Self::RecommendOption,
            ActionType::Relocate => Self::Relocate,
            ActionType::Reset => Self::Reset,
            ActionType::Samp => Self::Samp,
            ActionType::SBr => Self::SBr,
            ActionType::Script => Self::Script,
            ActionType::Send => Self::Link(SendArgs::try_from(scanner)?.into()),
            ActionType::SetOption => Self::SetOption,
            ActionType::Small => Self::Small,
            ActionType::Sound => {
                let sound = Sound::try_from(scanner)?;
                if sound.is_off() {
                    Self::SoundOff
                } else {
                    Self::Sound(sound)
                }
            }
            ActionType::Stat => Self::Stat,
            ActionType::Strikeout => Self::Strikeout,
            ActionType::Support => {
                let SupportArgs { supported } = scanner.try_into()?;
                Self::Support { supported }
            }
            ActionType::Tt => Self::Tt,
            ActionType::Ul => Self::Ul,
            ActionType::Underline => Self::Underline,
            ActionType::User => Self::User,
            ActionType::Var => {
                let VarArgs { variable, keywords } = scanner.try_into()?;
                Self::Var { variable, keywords }
            }
            ActionType::Version => Self::Version,
        })
    }
}
