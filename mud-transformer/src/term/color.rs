use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::{array, fmt};

use bytetable::ByteTable;
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
    palette: ByteTable<RgbColor>,
    defaults: [RgbColor; 16],
}

impl XTermPalette {
    /// Constructs a palette using [XTerm defaults](RgbColor::XTERM_256).
    pub const fn new() -> Self {
        Self {
            palette: ByteTable::new(RgbColor::XTERM_256),
            defaults: RgbColor::XTERM_16,
        }
    }

    /// Constructs a palette inside a box using [XTerm defaults](RgbColor::XTERM_256).
    pub fn new_boxed() -> Box<Self> {
        Box::default()
    }

    /// Borrows RGB colors as an array.
    #[inline]
    pub const fn as_array(&self) -> &[RgbColor; 256] {
        self.palette.as_array()
    }

    /// Mutably borrows RGB colors as an array.
    #[inline]
    pub const fn as_array_mut(&mut self) -> &mut [RgbColor; 256] {
        self.palette.as_array_mut()
    }

    /// Borrows RGB colors as a slice.
    #[inline]
    pub const fn as_slice(&self) -> &[RgbColor] {
        self.palette.as_slice()
    }

    /// Mutably borrows RGB colors as a slice.
    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [RgbColor] {
        self.palette.as_mut_slice()
    }

    /// Iterator that visits every XTerm color in order. The iterator element type is `RgbColor`.
    #[inline]
    pub fn iter(&self) -> array::IntoIter<RgbColor, 256> {
        self.into_iter()
    }

    /// Resets a color to its default value.
    pub fn reset_color(&mut self, i: u8) {
        self[i] = match self.defaults.get(i as usize) {
            Some(default) => *default,
            None => RgbColor::xterm(i),
        }
    }

    /// Resets all colors to their default values.
    pub fn reset(&mut self) {
        self.palette = ByteTable::new(RgbColor::XTERM_256);
        self.palette[..16].copy_from_slice(&self.defaults);
    }

    pub fn set_defaults(&mut self, defaults: &[RgbColor]) {
        let defaults: [RgbColor; 16] = if let Some(defaults) = defaults.first_chunk() {
            *defaults
        } else {
            let mut new_defaults = RgbColor::XTERM_16;
            new_defaults[..defaults.len()].copy_from_slice(defaults);
            new_defaults
        };
        for ((palette, old), new) in self.palette.iter_mut().zip(&self.defaults).zip(&defaults) {
            if *palette == *old {
                *palette = *new;
            }
        }
        self.defaults = defaults;
    }

    pub fn clear_defaults(&mut self) {
        self.set_defaults(&[]);
    }
}

impl Default for XTermPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for XTermPalette {
    type Target = ByteTable<RgbColor>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.palette
    }
}

impl DerefMut for XTermPalette {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.palette
    }
}

impl IntoIterator for &XTermPalette {
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
