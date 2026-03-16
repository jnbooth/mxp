use std::borrow::Cow;
use std::str::FromStr;

use super::action_kind::ActionKind;
use super::atomic_tag::AtomicTag;
use crate::arguments::{ArgumentScanner, Arguments, ExpectArg as _};
use crate::elements::{
    Color, Dest, Expire, Filter, Font, Frame, Gauge, Heading, Hyperlink, Image, Music, Relocate,
    Send, Sound, Stat, StyleVersion, Support, Var,
};
use crate::parse::{FromStrError, IntoOwnedString, Words};
use crate::{Error, ErrorKind};

/// Effect caused by an element. Created by applying [`Arguments`] to an element [`AtomicTag`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action<S = String> {
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
    /// [`<A>`](https://www.zuggsoft.com/zmud/mxp.htm#Links):
    /// Hyperlink.
    Hyperlink(Hyperlink<S>),
    /// [`<IMAGE>`](https://www.zuggsoft.com/zmud/mxp.htm#Images):
    /// Display an image.
    Image(Image<S>),
    /// [`<ITALIC>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
    /// Make text italic.
    Italic,
    /// [`<MUSIC>`](https://www.zuggsoft.com/zmud/mxp.htm#MSP%20Compatibility):
    /// Play music.
    Music(Music<S>),
    /// [`<MUSIC OFF>`](https://www.zuggsoft.com/zmud/mxp.htm#MSP%20Compatibility):
    /// Stop music.
    MusicOff,
    /// [`<MXP OFF>`](https://gpascal.com/forum/?id=232):
    /// MXP control command. This is an unofficial extension to the MXP protocol.
    MxpOff,
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
    /// [`<Send>`](https://www.zuggsoft.com/zmud/mxp.htm#Links):
    /// Turn text into a link that sends a command to the world.
    Send(Send<S>),
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
    /// Prompt client to respond with its client and version of MXP.
    Version,
}

impl<'a> Action<Cow<'a, str>> {
    pub(crate) fn decode<A>(action: ActionKind, mut scanner: A) -> crate::Result<Self>
    where
        A: ArgumentScanner<Output = Cow<'a, str>>,
    {
        Ok(match action {
            ActionKind::Bold => Self::Bold,
            ActionKind::Br => Self::Br,
            ActionKind::Color => Self::Color(Color::scan(scanner)?),
            ActionKind::Dest => Self::Dest(Dest::scan(scanner)?),
            ActionKind::Expire => Self::Expire(Expire::scan(scanner)?),
            ActionKind::Filter => Self::Filter(Filter::scan(scanner)?),
            ActionKind::Font => Self::Font(Font::scan(scanner)?),
            ActionKind::Frame => Self::Frame(Frame::scan(scanner)?),
            ActionKind::Gauge => Self::Gauge(Gauge::scan(scanner)?),
            ActionKind::H1 => Self::Heading(Heading::H1),
            ActionKind::H2 => Self::Heading(Heading::H2),
            ActionKind::H3 => Self::Heading(Heading::H3),
            ActionKind::H4 => Self::Heading(Heading::H4),
            ActionKind::H5 => Self::Heading(Heading::H5),
            ActionKind::H6 => Self::Heading(Heading::H6),
            ActionKind::Highlight => Self::Highlight,
            ActionKind::Hr => Self::Hr,
            ActionKind::Hyperlink => Self::Hyperlink(Hyperlink::scan(scanner)?),
            ActionKind::Image => Self::Image(Image::scan(scanner)?),
            ActionKind::Italic => Self::Italic,
            ActionKind::Mxp => {
                let command = scanner.decode_next()?.expect_some("off")?;
                if command.eq_ignore_ascii_case("off") {
                    Self::MxpOff
                } else {
                    return Err(Error::new(command, ErrorKind::UnexpectedArgument));
                }
            }
            ActionKind::Music => {
                let music = Music::scan(scanner)?;
                if music.is_off() {
                    Self::MusicOff
                } else {
                    Self::Music(music)
                }
            }
            ActionKind::NoBr => Self::NoBr,
            ActionKind::P => Self::P,
            ActionKind::Password => Self::Password,
            ActionKind::Relocate => Self::Relocate(Relocate::scan(scanner)?),
            ActionKind::Reset => Self::Reset,
            ActionKind::SBr => Self::SBr,
            ActionKind::Send => Self::Send(Send::scan(scanner)?),
            ActionKind::Small => Self::Small,
            ActionKind::Sound => {
                let sound = Sound::scan(scanner)?;
                if sound.is_off() {
                    Self::SoundOff
                } else {
                    Self::Sound(sound)
                }
            }
            ActionKind::Stat => Self::Stat(Stat::scan(scanner)?),
            ActionKind::Strikeout => Self::Strikeout,
            ActionKind::Support => Self::Support(Support::scan(scanner)?),
            ActionKind::Tt => Self::Tt,
            ActionKind::Underline => Self::Underline,
            ActionKind::User => Self::User,
            ActionKind::Var => Self::Var(Var::scan(scanner)?),
            ActionKind::Version => {
                if let Some(styleversion) = scanner.decode_next()? {
                    Self::StyleVersion(StyleVersion { styleversion })
                } else {
                    Self::Version
                }
            }
        })
    }
}

impl<S: IntoOwnedString> Action<S> {
    pub fn into_owned(self) -> Action<String> {
        self.map_text(IntoOwnedString::into_owned_string)
    }
}

impl<S> Action<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<F, T>(self, f: F) -> Action<T>
    where
        F: FnMut(S) -> T,
    {
        match self {
            Self::Bold => Action::Bold,
            Self::Br => Action::Br,
            Self::Color(color) => Action::Color(color),
            Self::Dest(dest) => Action::Dest(dest.map_text(f)),
            Self::Expire(expire) => Action::Expire(expire.map_text(f)),
            Self::Filter(filter) => Action::Filter(filter.map_text(f)),
            Self::Font(font) => Action::Font(font.map_text(f)),
            Self::Frame(frame) => Action::Frame(frame.map_text(f)),
            Self::Gauge(gauge) => Action::Gauge(gauge.map_text(f)),
            Self::Heading(heading) => Action::Heading(heading),
            Self::Highlight => Action::Highlight,
            Self::Hr => Action::Hr,
            Self::Hyperlink(hyperlink) => Action::Hyperlink(hyperlink.map_text(f)),
            Self::Image(image) => Action::Image(image.map_text(f)),
            Self::Italic => Action::Italic,
            Self::Music(music) => Action::Music(music.map_text(f)),
            Self::MusicOff => Action::MusicOff,
            Self::MxpOff => Action::MxpOff,
            Self::NoBr => Action::NoBr,
            Self::P => Action::P,
            Self::Password => Action::Password,
            Self::Relocate(relocate) => Action::Relocate(relocate.map_text(f)),
            Self::Reset => Action::Reset,
            Self::SBr => Action::SBr,
            Self::Small => Action::Small,
            Self::Send(send) => Action::Send(send.map_text(f)),
            Self::Sound(sound) => Action::Sound(sound.map_text(f)),
            Self::SoundOff => Action::SoundOff,
            Self::Stat(stat) => Action::Stat(stat.map_text(f)),
            Self::Strikeout => Action::Strikeout,
            Self::StyleVersion(style_version) => Action::StyleVersion(style_version.map_text(f)),
            Self::Support(support) => Action::Support(support.map_text(f)),
            Self::Tt => Action::Tt,
            Self::Underline => Action::Underline,
            Self::User => Action::User,
            Self::Var(var) => Action::Var(var.map_text(f)),
            Self::Version => Action::Version,
        }
    }
}

impl FromStr for Action<String> {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = crate::parse::cleanup_source(s)?;
        let mut words = Words::new(s);
        let name = words.next_or(ErrorKind::EmptyElement)?;
        crate::validate(name, ErrorKind::InvalidElementName)?;
        let tag = AtomicTag::well_known(name)
            .ok_or_else(|| FromStrError::UnexpectedTag(name.to_owned()))?;
        let args: Arguments<Cow<str>> = words.try_into()?;
        tag.check_arguments(&args)?;
        Ok(Action::decode(tag.action, args.scan(()))?.into_owned())
    }
}
