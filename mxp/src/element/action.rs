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
    /// AFK - away from keyboard time
    Afk,
    /// bold
    Bold,
    /// Hard Line break (secure)
    Br,
    /// Centre text
    Center,
    /// eg. <color fore=red back=blue>
    Color,
    /// destination frame
    Dest,
    /// expire
    Expire,
    /// sound/image filter
    Filter,
    /// Font appearance
    Font,
    /// frame
    Frame,
    /// gauge
    Gauge,
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
    /// Highlight text
    High,
    /// Horizontal rule (secure)
    Hr,
    /// Hyperlink (secure)
    Hyperlink,
    /// show image
    Image,
    /// italic
    Italic,
    /// List item
    Li,
    /// play music
    Music,
    /// MXP command (eg. MXP OFF)
    Mxp,
    /// ignore next newline
    NoBr,
    /// Ordered list
    Ol,
    /// Paragraph break (secure)
    P,
    /// send password
    Password,
    /// server sets option
    RecommendOption,
    /// causes a new connect to open
    Relocate,
    /// close all open tags
    Reset,
    /// Sample text
    Samp,
    /// Soft line break
    SBr,
    /// Client script (secure)
    Script,
    /// eg. <send href="go west"> west
    Send,
    /// client options set
    SetOption,
    /// Small text
    Small,
    /// play sound
    Sound,
    /// status
    Stat,
    /// Strikethrough
    Strikeout,
    /// what commands we support
    Support,
    /// Non-proportional font
    Tt,
    /// Unordered list
    Ul,
    /// underline
    Underline,
    /// send username
    User,
    /// Set variable
    Var,
    /// version request
    Version,
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
    /// AFK - away from keyboard time
    Afk { challenge: Option<S> },
    /// bold
    Bold,
    /// Hard Line break (secure)
    Br,
    /// Centre text
    Center,
    /// eg. <color fore=red back=blue>
    Color {
        fore: Option<RgbColor>,
        back: Option<RgbColor>,
    },
    /// destination frame
    Dest { name: S },
    /// expire
    Expire { name: Option<S> },
    /// sound/image filter
    Filter,
    /// font appearance
    Font(Font<S>),
    /// frame
    Frame(Frame<S>),
    /// gauge
    Gauge(Gauge<S>),
    /// heading (secure)
    Heading(Heading),
    /// Highlight text
    High,
    /// Horizontal rule (secure)
    Hr,
    /// show image
    Image(Image<S>),
    /// italic
    Italic,
    /// List item
    Li,
    /// Hyperlink or send prompt (secure)
    Link(Link),
    /// play music
    Music(Music<S>),
    /// stop all music
    MusicOff,
    /// MXP command (eg. MXP OFF)
    Mxp { keywords: EnumSet<MxpKeyword> },
    /// ignore next newline
    NoBr,
    /// Ordered list
    Ol,
    /// Paragraph break (secure)
    P,
    /// send password
    Password,
    /// server sets option
    RecommendOption,
    /// causes a new connect to open
    Relocate,
    /// close all open tags
    Reset,
    /// Sample text
    Samp,
    /// Soft linebreak
    SBr,
    /// Client script (secure)
    Script,
    /// client options set
    SetOption,
    /// Small text
    Small,
    /// play sound
    Sound(Sound<S>),
    /// stop all sound
    SoundOff,
    /// status
    Stat(Stat<S>),
    /// Strikethrough
    Strikeout,
    /// what commands we support
    Support { questions: Vec<S> },
    /// Non-proportional font
    Tt,
    /// Unordered list
    Ul,
    /// underline
    Underline,
    /// send username
    User,
    /// Set variable
    Var {
        variable: S,
        keywords: EnumSet<EntityKeyword>,
    },
    /// version request
    Version,
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
