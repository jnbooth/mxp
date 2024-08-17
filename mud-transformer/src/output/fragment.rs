use std::fmt::{self, Display, Formatter};
use std::num::NonZeroU8;
use std::vec;

use bytes::Bytes;
use enumeration::EnumSet;

use super::shared_string::SharedString;
use super::span::TextStyle;
use mxp::RgbColor;

pub type OutputDrain<'a> = vec::Drain<'a, Output>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Output {
    pub fragment: OutputFragment,
    pub gag: bool,
    pub window: Option<String>,
}

impl<T: Into<OutputFragment>> From<T> for Output {
    fn from(value: T) -> Self {
        Self {
            fragment: value.into(),
            gag: false,
            window: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutputFragment {
    Effect(EffectFragment),
    Frame(mxp::Frame),
    Hr,
    Image(mxp::Image),
    LineBreak,
    MxpError(mxp::Error),
    MxpEntity(EntityFragment),
    PageBreak,
    Telnet(TelnetFragment),
    Text(TextFragment),
}

impl OutputFragment {
    pub const fn is_visual(&self) -> bool {
        match self {
            Self::Effect(effect) => effect.is_visual(),
            Self::Frame(_)
            | Self::Hr
            | Self::Image(_)
            | Self::LineBreak
            | Self::PageBreak
            | Self::Text(_) => true,
            _ => false,
        }
    }

    pub const fn is_newline(&self) -> bool {
        matches!(self, Self::Hr | Self::LineBreak | Self::PageBreak)
    }

    pub(super) const fn should_flush(&self) -> bool {
        match self {
            Self::Effect(effect) => effect.is_visual(),
            Self::Frame(_)
            | Self::Hr
            | Self::Image(_)
            | Self::LineBreak
            | Self::PageBreak
            | Self::Telnet(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityFragment {
    Set {
        name: String,
        value: String,
        publish: bool,
        is_variable: bool,
    },
    Unset {
        name: String,
        is_variable: bool,
    },
}

impl EntityFragment {
    pub fn entity(entry: &mxp::EntityEntry) -> Self {
        Self::new(entry, false)
    }

    pub fn variable(entry: &mxp::EntityEntry) -> Self {
        Self::new(entry, true)
    }

    fn new(entry: &mxp::EntityEntry, is_variable: bool) -> Self {
        match entry.value {
            Some(entity) => Self::Set {
                name: entry.name.to_owned(),
                value: entity.value.clone(),
                publish: entity.published,
                is_variable,
            },
            None => Self::Unset {
                name: entry.name.to_owned(),
                is_variable,
            },
        }
    }
}

impl From<EntityFragment> for OutputFragment {
    fn from(value: EntityFragment) -> Self {
        Self::MxpEntity(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EffectFragment {
    Backspace,
    Beep,
    CarriageReturn,
    EraseCharacter,
    EraseLine,
    ExpireLinks(Option<String>),
    Music(mxp::Music),
    MusicOff,
    Sound(mxp::Sound),
    SoundOff,
}

impl EffectFragment {
    pub const fn is_visual(&self) -> bool {
        matches!(
            self,
            Self::Backspace | Self::CarriageReturn | Self::EraseCharacter | Self::EraseLine
        )
    }
}

impl From<EffectFragment> for OutputFragment {
    fn from(value: EffectFragment) -> Self {
        Self::Effect(value)
    }
}

impl From<mxp::Error> for OutputFragment {
    fn from(value: mxp::Error) -> Self {
        Self::MxpError(value)
    }
}

impl From<mxp::Frame> for OutputFragment {
    fn from(value: mxp::Frame) -> Self {
        Self::Frame(value)
    }
}

impl From<mxp::Image> for OutputFragment {
    fn from(value: mxp::Image) -> Self {
        Self::Image(value)
    }
}

impl From<mxp::Music> for OutputFragment {
    fn from(value: mxp::Music) -> Self {
        Self::Effect(EffectFragment::Music(value))
    }
}

impl From<mxp::Sound> for OutputFragment {
    fn from(value: mxp::Sound) -> Self {
        Self::Effect(EffectFragment::Sound(value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TelnetFragment {
    Afk { challenge: SharedString },
    Do { code: u8 },
    IacGa,
    Naws,
    Subnegotiation { code: u8, data: Bytes },
    Will { code: u8 },
}

impl From<TelnetFragment> for OutputFragment {
    fn from(value: TelnetFragment) -> Self {
        Self::Telnet(value)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextFragment {
    pub text: SharedString,
    pub flags: EnumSet<TextStyle>,
    pub foreground: RgbColor,
    pub background: RgbColor,
    pub font: Option<String>,
    pub size: Option<NonZeroU8>,
    pub action: Option<mxp::Link>,
    pub heading: Option<mxp::Heading>,
}

impl From<TextFragment> for OutputFragment {
    fn from(value: TextFragment) -> Self {
        Self::Text(value)
    }
}

impl Display for TextFragment {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("\x1B[")?;
        let fg = self.foreground;
        write!(f, "\x1B[38;2;{};{};{}", fg.r, fg.g, fg.b)?;
        let bg = self.background;
        if bg != RgbColor::BLACK {
            write!(f, ";48;2;{};{};{}", bg.r, bg.g, bg.b)?;
        }
        for flag in self.flags {
            if let Some(ansi) = flag.ansi() {
                write!(f, ";{ansi}")?;
            }
        }
        f.write_str("m")?;
        self.text.fmt(f)?;
        f.write_str("\x1B[0m")
    }
}
