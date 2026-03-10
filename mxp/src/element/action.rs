use std::borrow::Cow;

use super::action_kind::ActionKind;
use crate::argument::{Decoder, Scan};
use crate::elements::{
    Color, Dest, Expire, Filter, Font, Frame, Gauge, Heading, Hyperlink, Image, Link, Music, Mxp,
    Relocate, Send, Sound, Stat, StyleVersion, Support, Var,
};

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
    Color(Color),
    /// [`<DEST>`](https://www.zuggsoft.com/zmud/mxp.htm#Cursor%20Control):
    /// Set destination frame.
    Dest(Dest<S>),
    /// [`<EXPIRE>`](https://www.zuggsoft.com/zmud/mxp.htm#Links):
    /// Expire links.
    Expire(Expire<S>),
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
    Mxp(Mxp),
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
    /// [`<VERSION>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control):
    /// The client should cache this style-sheet version number and return it when
    /// requested by a plain `<VERSION>` request.
    StyleVersion(StyleVersion<S>),
    /// [`<SUPPORT>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control):
    /// Prompt client to respond with the commands that it supports.
    Support(Support<S>),
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
    Var(Var<S>),
    /// [`<VERSION>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control):
    // Prompt client to respond with its client and version of MXP.
    Version,
}

impl<'a> Action<Cow<'a, str>> {
    pub(crate) fn decode<D>(action: ActionKind, mut scanner: Scan<'a, D>) -> crate::Result<Self>
    where
        D: Decoder,
    {
        Ok(match action {
            ActionKind::Bold => Self::Bold,
            ActionKind::Br => Self::Br,
            ActionKind::Color => Self::Color(scanner.try_into()?),
            ActionKind::Dest => Self::Dest(scanner.try_into()?),
            ActionKind::Expire => Self::Expire(scanner.try_into()?),
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
            ActionKind::Hyperlink => Self::Link(Hyperlink::try_from(scanner)?.into()),
            ActionKind::Image => Self::Image(Image::try_from(scanner)?),
            ActionKind::Italic => Self::Italic,
            ActionKind::Mxp => Self::Mxp(scanner.try_into()?),
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
            ActionKind::Send => Self::Link(Send::try_from(scanner)?.into()),
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
            ActionKind::Support => Self::Support(scanner.try_into()?),
            ActionKind::Tt => Self::Tt,
            ActionKind::Underline => Self::Underline,
            ActionKind::User => Self::User,
            ActionKind::Var => Self::Var(scanner.try_into()?),
            ActionKind::Version => {
                if let Some(styleversion) = scanner.next()? {
                    Self::StyleVersion(StyleVersion { styleversion })
                } else {
                    Self::Version
                }
            }
        })
    }
}
