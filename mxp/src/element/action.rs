use std::borrow::Cow;

use flagset::{FlagSet, flags};

use super::bar::{Gauge, Stat};
use super::filter::Filter;
use super::font::Font;
use super::frame::{DestArgs, Frame};
use super::image::Image;
use super::link::{ExpireArgs, HyperlinkArgs, Link, SendArgs};
use super::relocate::Relocate;
use super::sound::{Music, Sound};
use crate::argument::args::{ColorArgs, MxpArgs, SupportArgs, VarArgs, VersionArgs};
use crate::argument::{Decoder, Scan};
use crate::color::RgbColor;
use crate::keyword::{EntityKeyword, MxpKeyword};

flags! {
    pub enum ActionKind: u64 {
        /// bold
        Bold,
        /// Hard Line break (secure)
        Br,
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
        Highlight,
        /// Horizontal rule (secure)
        Hr,
        /// Hyperlink (secure)
        Hyperlink,
        /// show image
        Image,
        /// italic
        Italic,
        /// play music
        Music,
        /// MXP command (eg. MXP OFF)
        Mxp,
        /// ignore next newline
        NoBr,
        /// Paragraph break (secure)
        P,
        /// send password
        Password,
        /// causes a new connect to open
        Relocate,
        /// close all open tags
        Reset,
        /// Soft line break
        SBr,
        /// eg. <send href="go west"> west
        Send,
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
        /// underline
        Underline,
        /// send username
        User,
        /// Set variable
        Var,
        /// version request
        Version,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Heading {
    H1 = 1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl Heading {
    /// # Examples
    ///
    /// ```
    /// assert_eq!(mxp::Heading::H1.level(), 1);
    /// assert_eq!(mxp::Heading::H5.level(), 5);
    /// ```
    pub const fn level(self) -> u8 {
        self as u8
    }
}

impl ActionKind {
    /// Returns `true` if this is a command tag, i.e. a tag with no closing tag.
    pub const fn is_command(self) -> bool {
        matches!(
            self,
            Self::Br
                | Self::Expire
                | Self::Filter
                | Self::Gauge
                | Self::Hr
                | Self::Music
                | Self::Mxp
                | Self::NoBr
                | Self::Password
                | Self::Relocate
                | Self::Reset
                | Self::SBr
                | Self::Stat
                | Self::Support
                | Self::User
                | Self::Version
                | Self::Frame
                | Self::Image
                | Self::Sound
        )
    }

    /// Returns `true` if the action can be used if the MXP [`Mode`](crate::Mode) is "open".
    pub const fn is_open(self) -> bool {
        matches!(
            self,
            Self::Bold
                | Self::Color
                | Self::Italic
                | Self::Highlight
                | Self::Strikeout
                | Self::Small
                | Self::Tt
                | Self::Underline
                | Self::Font
        )
    }
}

/// Effect caused by an [`Element`](crate::Element).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action<S> {
    /// [`<BOLD>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
    /// Make text bold.
    Bold,
    /// [`<BR>`](https://www.zuggsoft.com/zmud/mxp.htm#Line%20Spacing):
    /// Insert a hard line break.
    Br,
    ///[`<COLOR>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
    /// Change text color.
    Color {
        fore: Option<RgbColor>,
        back: Option<RgbColor>,
    },
    /// [`<DEST>`](https://www.zuggsoft.com/zmud/mxp.htm#Cursor%20Control):
    /// Set destination frame.
    Dest { name: S },
    /// [`<EXPIRE>`](https://www.zuggsoft.com/zmud/mxp.htm#Links):
    /// Expire links.
    Expire { name: Option<S> },
    /// [`<FILTER>`](https://www.zuggsoft.com/zmud/mxp.htm#File%20Filters):
    /// Set file filter.
    Filter(Filter<S>),
    /// [`<FONT>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
    /// Change text font.
    Font(Font<S>),
    /// [`<FRAME>`](https://www.zuggsoft.com/zmud/mxp.htm#Frames):
    /// Create a frame window.
    Frame(Frame<S>),
    /// [`<GAUGE>`](https://www.zuggsoft.com/zmud/mxp.htm#Using%20Entities):
    /// Display an MXP entity value as a gauge.
    Gauge(Gauge<S>),
    /// [`<H{N}>`](https://www.zuggsoft.com/zmud/mxp.htm#HTML%20tags):
    /// Format text as a heading.
    Heading(Heading),
    /// [`<HIGH>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
    /// Highlight text.
    Highlight,
    /// [`<HR>`](https://www.zuggsoft.com/zmud/mxp.htm#HTML%20tags):
    /// Insert a horizontal rule.
    Hr,
    /// [`<IMAGE>`](https://www.zuggsoft.com/zmud/mxp.htm#Images):
    /// Display an image.
    Image(Image<S>),
    /// [`<ITALIC>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
    /// Make text italic.
    Italic,
    /// [`<SEND>`, `<A>`](https://www.zuggsoft.com/zmud/mxp.htm#Links):
    /// Hyperlink or send prompt.
    Link(Link),
    /// [`<MUSIC>`](https://www.zuggsoft.com/zmud/mxp.htm#MSP%20Compatibility):
    /// Play music.
    Music(Music<S>),
    /// [`<MUSIC OFF>`](https://www.zuggsoft.com/zmud/mxp.htm#MSP%20Compatibility):
    /// Stop music.
    MusicOff,
    /// [`<MXP>`](https://gpascal.com/forum/?id=232):
    /// MXP control command. This is an unofficial extension to the MXP protocol.
    Mxp { keywords: FlagSet<MxpKeyword> },
    /// [`<NOBR>`](https://www.zuggsoft.com/zmud/mxp.htm#Line%20Spacing):
    /// Ignore next newline.
    NoBr,
    /// [`<P>`](https://www.zuggsoft.com/zmud/mxp.htm#Line%20Spacing):
    /// Insert a paragraph break.
    P,
    /// [`<PASSWORD>`](https://www.zuggsoft.com/zmud/mxp.htm#Crosslinking%20multiple%20MUD%20servers):
    /// Prompt client to send user password.
    Password,
    /// [`<RELOCATE>`](https://www.zuggsoft.com/zmud/mxp.htm#Crosslinking%20multiple%20MUD%20servers):
    /// Prompt client to switch to a new network connection.
    Relocate(Relocate<S>),
    /// [`<RESET>`](https://gpascal.com/forum/?id=232):
    /// Close all open tags. This is an unofficial extension to the MXP protocol.
    Reset,
    /// [`<SBR>`](https://www.zuggsoft.com/zmud/mxp.htm#Line%20Spacing):
    /// Insert a soft linebreak.
    SBr,
    /// [`<SMALL>`](https://www.zuggsoft.com/zmud/mxp.htm#HTML%20tags):
    /// Display text in a smaller size.
    Small,
    /// [`<SOUND>`](https://www.zuggsoft.com/zmud/mxp.htm#MSP%20Compatibility):
    /// Play a sound file.
    Sound(Sound<S>),
    /// [`<SOUND OFF>`](https://www.zuggsoft.com/zmud/mxp.htm#MSP%20Compatibility):
    /// Stop all sound.
    SoundOff,
    /// [`<STAT>`](https://www.zuggsoft.com/zmud/mxp.htm#Using%20Entities):
    /// Display an MXP entity value on the status bar.
    Stat(Stat<S>),
    /// [`<STRIKEOUT>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
    /// Strike-out the text.
    Strikeout,
    /// [`<SUPPORT>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control):
    /// Prompt client to respond with the commands that it supports.
    Support { questions: Vec<S> },
    /// [`<TT>`](https://www.zuggsoft.com/zmud/mxp.htm#HTML%20tags):
    /// Display text in a non-proportional font.
    Tt,
    /// [`<UNDERLINE>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
    /// Underline text.
    Underline,
    /// [`<USER>`](https://www.zuggsoft.com/zmud/mxp.htm#Crosslinking%20multiple%20MUD%20servers):
    /// Prompt client to send username.
    User,
    /// [`<VAR>`](https://www.zuggsoft.com/zmud/mxp.htm#ENTITY):
    /// Set an MXP variable.
    Var {
        variable: S,
        keywords: FlagSet<EntityKeyword>,
    },
    /// [`<VERSION>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control):
    /// If `styleversion` is `None`, prompt client to respond with its client and version of MXP.
    /// Per the specification, the MUD server can alternatively send `<VERSION styleversion>`.
    /// In this case, the client should cache this style-sheet version number and return it when
    /// requested by a plain `<VERSION>` request.
    Version { styleversion: Option<S> },
}

impl<'a> Action<Cow<'a, str>> {
    pub(crate) fn parse<D, S>(action: ActionKind, scanner: Scan<'a, D, S>) -> crate::Result<Self>
    where
        D: Decoder,
        S: AsRef<str>,
    {
        Ok(match action {
            ActionKind::Bold => Self::Bold,
            ActionKind::Br => Self::Br,
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
            ActionKind::Filter => Self::Filter(scanner.try_into()?),
            ActionKind::Font => Self::Font(scanner.try_into()?),
            ActionKind::Frame => Self::Frame(scanner.try_into()?),
            ActionKind::Gauge => Self::Gauge(scanner.try_into()?),
            ActionKind::H1 => Self::Heading(Heading::H1),
            ActionKind::H2 => Self::Heading(Heading::H2),
            ActionKind::H3 => Self::Heading(Heading::H3),
            ActionKind::H4 => Self::Heading(Heading::H4),
            ActionKind::H5 => Self::Heading(Heading::H5),
            ActionKind::H6 => Self::Heading(Heading::H6),
            ActionKind::Highlight => Self::Highlight,
            ActionKind::Hr => Self::Hr,
            ActionKind::Hyperlink => Self::Link(HyperlinkArgs::try_from(scanner)?.into()),
            ActionKind::Image => Self::Image(Image::try_from(scanner)?),
            ActionKind::Italic => Self::Italic,
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
            ActionKind::P => Self::P,
            ActionKind::Password => Self::Password,
            ActionKind::Relocate => Self::Relocate(scanner.try_into()?),
            ActionKind::Reset => Self::Reset,
            ActionKind::SBr => Self::SBr,
            ActionKind::Send => Self::Link(SendArgs::try_from(scanner)?.into()),
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
            ActionKind::Underline => Self::Underline,
            ActionKind::User => Self::User,
            ActionKind::Var => {
                let VarArgs { variable, keywords } = scanner.try_into()?;
                Self::Var { variable, keywords }
            }
            ActionKind::Version => {
                let VersionArgs { styleversion } = scanner.try_into()?;
                Self::Version { styleversion }
            }
        })
    }
}
