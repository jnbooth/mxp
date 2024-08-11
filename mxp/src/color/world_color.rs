use std::fmt::{self, Display, Formatter};
use std::hash::Hash;

use super::rgb::RgbColor;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorldColor {
    Ansi(u8),
    Rgb(RgbColor),
}

#[allow(unused)]
impl WorldColor {
    pub const BLACK: Self = Self::Ansi(0);
    pub const RED: Self = Self::Ansi(1);
    pub const GREEN: Self = Self::Ansi(2);
    pub const YELLOW: Self = Self::Ansi(3);
    pub const BLUE: Self = Self::Ansi(4);
    pub const PURPLE: Self = Self::Ansi(5);
    pub const CYAN: Self = Self::Ansi(6);
    pub const WHITE: Self = Self::Ansi(7);
    pub const BRIGHT_BLACK: Self = Self::Ansi(8);
    pub const BRIGHT_RED: Self = Self::Ansi(9);
    pub const BRIGHT_GREEN: Self = Self::Ansi(10);
    pub const BRIGHT_YELLOW: Self = Self::Ansi(11);
    pub const BRIGHT_BLUE: Self = Self::Ansi(12);
    pub const BRIGHT_PURPLE: Self = Self::Ansi(13);
    pub const BRIGHT_CYAN: Self = Self::Ansi(14);
    pub const BRIGHT_WHITE: Self = Self::Ansi(15);
}

impl Default for WorldColor {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Display for WorldColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ansi(code) => write!(f, "Ansi({code})"),
            Self::Rgb(color) => color.fmt(f),
        }
    }
}

impl From<RgbColor> for WorldColor {
    fn from(value: RgbColor) -> Self {
        Self::Rgb(value)
    }
}

impl From<WorldColor> for RgbColor {
    fn from(value: WorldColor) -> Self {
        match value {
            WorldColor::Ansi(code) => RgbColor::xterm(code),
            WorldColor::Rgb(color) => color,
        }
    }
}
