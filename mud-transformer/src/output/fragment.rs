use std::fmt::{self, Display, Formatter};
use std::vec;

use bytes::Bytes;
use enumeration::{Enum, EnumSet};

use crate::escape::ansi;

use super::span::{Heading, TextStyle};
use mxp::{HexColor, WorldColor};

pub type OutputDrain<'a> = vec::Drain<'a, OutputFragment>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutputFragment {
    Effect(EffectFragment),
    Hr,
    Image(String),
    LineBreak,
    PageBreak,
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

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextFragment {
    pub text: Bytes,
    pub flags: EnumSet<TextStyle>,
    pub foreground: WorldColor,
    pub background: WorldColor,
    pub action: Option<mxp::Link>,
    pub heading: Option<Heading>,
    /// Which variable to set (FLAG in MXP).
    pub variable: Option<String>,
}

impl AsRef<[u8]> for TextFragment {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.text
    }
}

impl Display for TextFragment {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "\x1B[")?;
        match self.foreground {
            WorldColor::Ansi(code) => write!(f, "\x1B[{}", code + ansi::FG_BLACK),
            WorldColor::Hex(color) => {
                write!(f, "\x1B[38;2;{};{};{}", color.r(), color.g(), color.b())
            }
        }?;
        match self.background {
            WorldColor::Ansi(0) => Ok(()),
            WorldColor::Hex(HexColor::BLACK) => Ok(()),
            WorldColor::Ansi(code) => write!(f, ";{}", code + ansi::BG_BLACK),
            WorldColor::Hex(color) => write!(f, ";48;2;{};{};{}", color.r(), color.g(), color.b()),
        }?;
        for flag in self.flags {
            if let Some(ansi) = flag.ansi() {
                write!(f, ";{ansi}")?;
            }
        }
        f.write_str("m")?;
        String::from_utf8_lossy(&self.text).fmt(f)?;
        f.write_str("\x1B[0m")
    }
}
