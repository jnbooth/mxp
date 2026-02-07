use std::hash::Hash;
use std::ops::{Index, IndexMut};
use std::{array, fmt, iter, slice};

use mxp::RgbColor;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A color set by the terminal.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum TermColor {
    #[default]
    Unset,
    /// 8-bit ANSI color code. Some clients allow users to customize the RGB output of the first
    /// 16 ANSI colors.
    Ansi(u8),
    /// 24-bit color.
    Rgb(RgbColor),
}

impl TermColor {
    /*
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
    */
}

impl From<RgbColor> for TermColor {
    fn from(value: RgbColor) -> Self {
        Self::Rgb(value)
    }
}

impl fmt::Display for TermColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unset => f.write_str("--"),
            Self::Ansi(n) => write!(f, "{n}"),
            Self::Rgb(color) => write!(f, "{color}"),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicColor {
    /// OSC 10 (Set Text Foreground Color)
    TextForeground = 10,
    /// OSC 11 (Set Text Background Color)
    TextBackground,
    /// OSC 12 (Set Text Cursor Color)
    TextCursor,
    /// OSC 13 (Set Mouse Foreground Color)
    MouseForeground,
    /// OSC 14 (Set Mouse Background Color)
    MouseBackground,
    /// OSC 15 (Set Tektronix Foreground Color)
    TektronixForeground,
    /// OSC 16 (Set Tektronix Background Color)
    TektronixBackground,
    /// OSC 17 (Set Highlight Color)
    Highlight,
    /// OSC 18 (Set Tetronix Cursor Color)
    TektronixCursor,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct XTermPalette([RgbColor; 256]);

impl XTermPalette {
    pub const fn new() -> Self {
        Self(*RgbColor::XTERM_256)
    }

    pub const fn reset(&mut self) {
        self.0 = *RgbColor::XTERM_256;
    }
}

impl Default for XTermPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<u8> for XTermPalette {
    type Output = RgbColor;

    #[inline]
    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u8> for XTermPalette {
    #[inline]
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl<'a> IntoIterator for &'a XTermPalette {
    type Item = RgbColor;

    type IntoIter = iter::Copied<slice::Iter<'a, RgbColor>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

impl IntoIterator for XTermPalette {
    type Item = RgbColor;

    type IntoIter = array::IntoIter<RgbColor, 256>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
