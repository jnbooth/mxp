use std::fmt::{self, Display, Formatter};
use std::vec;

use bytes::Bytes;
use enumeration::{Enum, EnumSet};

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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutputFragment {
    Effect(EffectFragment),
    Hr,
    Image(String),
    LineBreak,
    MxpError(mxp::Error),
    MxpVariable { name: String, value: Option<String> },
    PageBreak,
    Telnet(TelnetFragment),
    Text(TextFragment),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum EffectFragment {
    Backspace,
    Beep,
    CarriageReturn,
    EraseCharacter,
    EraseLine,
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
