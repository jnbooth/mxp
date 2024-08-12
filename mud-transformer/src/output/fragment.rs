use std::fmt::{self, Display, Formatter};
use std::vec;

use bytes::Bytes;
use enumeration::{Enum, EnumSet};

use super::shared_string::SharedString;
use super::span::TextStyle;
use mxp::escape::ansi;
use mxp::{RgbColor, TermColor};

pub type OutputDrain<'a> = vec::Drain<'a, OutputFragment>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutputFragment {
    Effect(EffectFragment),
    Hr,
    Image(String),
    LineBreak,
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
    pub foreground: TermColor,
    pub background: TermColor,
    pub action: Option<Box<mxp::Link>>,
    pub heading: Option<mxp::Heading>,
    /// Which variable to set (FLAG in MXP).
    pub variable: Option<String>,
}

impl From<TextFragment> for OutputFragment {
    fn from(value: TextFragment) -> Self {
        Self::Text(value)
    }
}

impl Display for TextFragment {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("\x1B[")?;
        match self.foreground {
            TermColor::Ansi(code) => write!(f, "\x1B[{}", code + ansi::FG_BLACK),
            TermColor::Rgb(color) => {
                write!(f, "\x1B[38;2;{};{};{}", color.r, color.g, color.b)
            }
        }?;
        match self.background {
            TermColor::Ansi(0) => Ok(()),
            TermColor::Rgb(RgbColor::BLACK) => Ok(()),
            TermColor::Ansi(code) => write!(f, ";{}", code + ansi::BG_BLACK),
            TermColor::Rgb(color) => write!(f, ";48;2;{};{};{}", color.r, color.g, color.b),
        }?;
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
