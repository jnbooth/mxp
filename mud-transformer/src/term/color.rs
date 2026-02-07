use std::hash::Hash;
use std::ops::{
    Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};
use std::{array, fmt};

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

/// Contains an OSC color palette of XTerm colors.
///
/// Note: This is a large struct (816 bytes). It should generally be boxed.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct XTermPalette {
    palette: [RgbColor; 256],
    defaults: [RgbColor; 16],
}

impl XTermPalette {
    /// Constructs a palette using [XTerm defaults](RgbColor::XTERM_256).
    pub const fn new() -> Self {
        Self {
            palette: RgbColor::XTERM_256,
            defaults: RgbColor::XTERM_16,
        }
    }

    /// Constructs a palette inside a box using [XTerm defaults](RgbColor::XTERM_256).
    pub fn new_boxed() -> Box<Self> {
        Box::default()
    }

    /// Returns the color set for the specified XTerm color index.
    /// 0 = black, 1 = maroon, etc.
    #[inline]
    pub const fn get(&self, i: u8) -> RgbColor {
        self.palette[i as usize]
    }

    /// Returns a mutable reference to the color set for the specified XTerm color index.
    /// 0 = black, 1 = maroon, etc.
    #[inline]
    pub const fn get_mut(&mut self, i: u8) -> &mut RgbColor {
        &mut self.palette[i as usize]
    }

    /// Iterates over the palette.
    #[inline]
    pub fn iter(&self) -> array::IntoIter<RgbColor, 256> {
        self.into_iter()
    }

    /// Resets a color to its default value.
    pub fn reset_color(&mut self, i: u8) {
        let i_usize = i as usize;
        self.palette[i_usize] = if i_usize < self.defaults.len() {
            self.defaults[i_usize]
        } else {
            RgbColor::xterm(i)
        }
    }

    /// Resets all colors to their default values.
    pub const fn reset(&mut self) {
        self.palette = RgbColor::XTERM_256;
        self.apply_defaults();
    }

    const fn is_default(&self, i: usize) -> bool {
        self.palette[i].code() == self.defaults[i].code()
    }

    pub const fn set_defaults(&mut self, defaults: &[RgbColor]) {
        let defaults_len = defaults.len();
        let min_len = if defaults_len > 16 { 16 } else { defaults_len };
        let mut i = 0;
        while i < min_len {
            if self.is_default(i) {
                self.palette[i] = defaults[i];
            }
            i += 1;
        }
        while i < 16 {
            if self.is_default(i) {
                self.palette[i] = RgbColor::XTERM_16[i];
            }
            i += 1;
        }
        self.defaults = RgbColor::XTERM_16;
        let (self_defaults, _) = self.defaults.split_at_mut(min_len);
        let (defaults, _) = defaults.split_at(min_len);
        self_defaults.copy_from_slice(defaults);
    }

    pub const fn clear_defaults(&mut self) {
        let mut i = 0;
        while i < 16 {
            if self.is_default(i) {
                self.palette[i] = RgbColor::XTERM_16[i];
            }
            i += 1;
        }
        self.defaults = RgbColor::XTERM_16;
    }

    const fn apply_defaults(&mut self) {
        self.palette
            .first_chunk_mut::<16>()
            .unwrap()
            .copy_from_slice(&self.defaults);
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
        &self.palette[index as usize]
    }
}

impl IndexMut<u8> for XTermPalette {
    #[inline]
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.palette[index as usize]
    }
}

impl Index<RangeFrom<u8>> for XTermPalette {
    type Output = [RgbColor];

    #[inline]
    fn index(&self, index: RangeFrom<u8>) -> &Self::Output {
        &self.palette[index.start as usize..]
    }
}

impl IndexMut<RangeFrom<u8>> for XTermPalette {
    #[inline]
    fn index_mut(&mut self, index: RangeFrom<u8>) -> &mut Self::Output {
        &mut self.palette[index.start as usize..]
    }
}

impl Index<RangeInclusive<u8>> for XTermPalette {
    type Output = [RgbColor];

    #[inline]
    fn index(&self, index: RangeInclusive<u8>) -> &Self::Output {
        let (start, end) = index.into_inner();
        &self.palette[start as usize..=end as usize]
    }
}

impl IndexMut<RangeInclusive<u8>> for XTermPalette {
    #[inline]
    fn index_mut(&mut self, index: RangeInclusive<u8>) -> &mut Self::Output {
        let (start, end) = index.into_inner();
        &mut self.palette[start as usize..=end as usize]
    }
}

impl Index<Range<u8>> for XTermPalette {
    type Output = [RgbColor];

    #[inline]
    fn index(&self, index: Range<u8>) -> &Self::Output {
        &self.palette[index.start as usize..index.end as usize]
    }
}

impl IndexMut<Range<u8>> for XTermPalette {
    #[inline]
    fn index_mut(&mut self, index: Range<u8>) -> &mut Self::Output {
        &mut self.palette[index.start as usize..index.end as usize]
    }
}

impl Index<RangeFull> for XTermPalette {
    type Output = [RgbColor];

    #[inline]
    fn index(&self, index: RangeFull) -> &Self::Output {
        &self.palette[index]
    }
}

impl IndexMut<RangeFull> for XTermPalette {
    #[inline]
    fn index_mut(&mut self, index: RangeFull) -> &mut Self::Output {
        &mut self.palette[index]
    }
}

impl Index<RangeTo<u8>> for XTermPalette {
    type Output = [RgbColor];

    #[inline]
    fn index(&self, index: RangeTo<u8>) -> &Self::Output {
        &self.palette[..index.end as usize]
    }
}

impl IndexMut<RangeTo<u8>> for XTermPalette {
    #[inline]
    fn index_mut(&mut self, index: RangeTo<u8>) -> &mut Self::Output {
        &mut self.palette[..index.end as usize]
    }
}

impl Index<RangeToInclusive<u8>> for XTermPalette {
    type Output = [RgbColor];

    #[inline]
    fn index(&self, index: RangeToInclusive<u8>) -> &Self::Output {
        &self.palette[..=index.end as usize]
    }
}

impl IndexMut<RangeToInclusive<u8>> for XTermPalette {
    #[inline]
    fn index_mut(&mut self, index: RangeToInclusive<u8>) -> &mut Self::Output {
        &mut self.palette[..index.end as usize]
    }
}

impl IntoIterator for &'_ XTermPalette {
    type Item = RgbColor;

    type IntoIter = array::IntoIter<RgbColor, 256>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.palette.into_iter()
    }
}

impl IntoIterator for XTermPalette {
    type Item = RgbColor;

    type IntoIter = array::IntoIter<RgbColor, 256>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.palette.into_iter()
    }
}
