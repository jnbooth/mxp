use super::hex_color::HexColor;
use super::xterm::xterm;
use crate::escape::ansi;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorldColor {
    Ansi(u8),
    Hex(HexColor),
}

impl From<HexColor> for WorldColor {
    fn from(value: HexColor) -> Self {
        Self::Hex(value)
    }
}

impl From<u8> for WorldColor {
    fn from(value: u8) -> Self {
        Self::Hex(xterm(value))
    }
}

impl From<u32> for WorldColor {
    fn from(value: u32) -> Self {
        Self::Hex(HexColor::new(value))
    }
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

    pub const fn fg_from_ansi(code: u8) -> Option<WorldColor> {
        if code == ansi::FG_DEFAULT {
            Some(Self::WHITE)
        } else if code >= ansi::FG_BLACK && code <= ansi::FG_WHITE {
            Some(Self::Ansi((code - ansi::FG_BLACK)))
        } else {
            None
        }
    }

    pub const fn bg_from_ansi(code: u8) -> Option<WorldColor> {
        if code == ansi::BG_DEFAULT {
            Some(Self::BLACK)
        } else if code >= ansi::BG_BLACK && code <= ansi::BG_WHITE {
            Some(Self::Ansi((code - ansi::BG_BLACK)))
        } else {
            None
        }
    }
}

impl Default for WorldColor {
    fn default() -> Self {
        Self::BLACK
    }
}
