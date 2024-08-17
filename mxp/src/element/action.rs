use super::bar::{Gauge, Stat};
use super::font::Font;
use super::frame::{DestArgs, Frame};
use super::image::Image;
use super::link::{ExpireArgs, HyperlinkArgs, Link, SendArgs};
use super::sound::{Music, Sound};
use crate::argument::args::{AfkArgs, ColorArgs, MxpArgs, SupportArgs, VarArgs};
use crate::argument::{Decoder, Scan};
use crate::color::RgbColor;
use crate::keyword::{EntityKeyword, MxpKeyword};
use enumeration::{Enum, EnumSet};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum ActionKind {
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

    /// gauge
    Gauge(Gauge<S>),
    /// status
    Stat(Stat<S>),
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
        questions: Vec<S>,
    },

    /// client options set
    SetOption,
    /// server sets option
    RecommendOption,
}

impl<S: AsRef<str>> Action<S> {
    pub fn new<'a, D: Decoder>(action: ActionKind, scanner: Scan<'a, D>) -> crate::Result<Self>
    where
        D: Decoder<Output<'a> = S>,
    {
        Ok(match action {
            ActionKind::Afk => {
                let AfkArgs { challenge } = scanner.try_into()?;
                Self::Afk { challenge }
            }
            ActionKind::Bold => Self::Bold,
            ActionKind::Br => Self::Br,
            ActionKind::Center => Self::Center,
            ActionKind::Color => {
                let ColorArgs { fore, back } = scanner.try_into()?;
                Self::Color { fore, back }
            }
            ActionKind::Dest => {
                let DestArgs { name } = scanner.try_into()?;
                Self::Dest { name }
            }
            ActionKind::Expire => {
                let ExpireArgs { name } = scanner.try_into()?;
                Self::Expire { name }
            }
            ActionKind::Filter => Self::Filter,
            ActionKind::Font => Self::Font(scanner.try_into()?),
            ActionKind::Frame => Self::Frame(scanner.try_into()?),
            ActionKind::Gauge => Self::Gauge(scanner.try_into()?),
            ActionKind::H1 => Self::Heading(Heading::H1),
            ActionKind::H2 => Self::Heading(Heading::H2),
            ActionKind::H3 => Self::Heading(Heading::H3),
            ActionKind::H4 => Self::Heading(Heading::H4),
            ActionKind::H5 => Self::Heading(Heading::H5),
            ActionKind::H6 => Self::Heading(Heading::H6),
            ActionKind::High => Self::High,
            ActionKind::Hr => Self::Hr,
            ActionKind::Hyperlink => Self::Link(HyperlinkArgs::try_from(scanner)?.into()),
            ActionKind::Image => Self::Image(Image::try_from(scanner)?),
            ActionKind::Italic => Self::Italic,
            ActionKind::Li => Self::Li,
            ActionKind::Mxp => {
                let MxpArgs { keywords } = scanner.try_into()?;
                Self::Mxp { keywords }
            }
            ActionKind::Music => {
                let music = Music::try_from(scanner)?;
                if music.is_off() {
                    Self::MusicOff
                } else {
                    Self::Music(music)
                }
            }
            ActionKind::NoBr => Self::NoBr,
            ActionKind::Ol => Self::Ol,
            ActionKind::P => Self::P,
            ActionKind::Password => Self::Password,
            ActionKind::RecommendOption => Self::RecommendOption,
            ActionKind::Relocate => Self::Relocate,
            ActionKind::Reset => Self::Reset,
            ActionKind::Samp => Self::Samp,
            ActionKind::SBr => Self::SBr,
            ActionKind::Script => Self::Script,
            ActionKind::Send => Self::Link(SendArgs::try_from(scanner)?.into()),
            ActionKind::SetOption => Self::SetOption,
            ActionKind::Small => Self::Small,
            ActionKind::Sound => {
                let sound = Sound::try_from(scanner)?;
                if sound.is_off() {
                    Self::SoundOff
                } else {
                    Self::Sound(sound)
                }
            }
            ActionKind::Stat => Self::Stat(scanner.try_into()?),
            ActionKind::Strikeout => Self::Strikeout,
            ActionKind::Support => {
                let SupportArgs { questions } = scanner.try_into()?;
                Self::Support { questions }
            }
            ActionKind::Tt => Self::Tt,
            ActionKind::Ul => Self::Ul,
            ActionKind::Underline => Self::Underline,
            ActionKind::User => Self::User,
            ActionKind::Var => {
                let VarArgs { variable, keywords } = scanner.try_into()?;
                Self::Var { variable, keywords }
            }
            ActionKind::Version => Self::Version,
        })
    }
}
