use casefold::ascii::CaseFoldMap;

use super::error::{HexOutOfRangeError, ParseHexColorError};
use super::fmt::RgbDigits;
use super::named::{NamedColorIter, NAMED_COLORS};
use super::xterm::{first_xterm_colors, XTERM_COLORS};
use std::fmt;
use std::str::{self, FromStr};
use std::sync::LazyLock;

/// A 24-bit color consisting of a red, green, and blue value.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const BLACK: Self = Self::hex(0x000000);
    pub const WHITE: Self = Self::hex(0xFFFFFF);

    /// Standard definitions for 3-bit color.
    pub const XTERM_8: &[Self; 8] = first_xterm_colors();

    /// Standard definitions for 4-bit color.
    pub const XTERM_16: &[Self; 16] = first_xterm_colors();

    /// Standard definitions for 8-bit color.
    pub const XTERM_256: &[Self; 256] = XTERM_COLORS;

    /// Constructs an `RgbColor` from a red, green, and blue value.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Constructs an `RgbColor` from a color hex code.
    /// The last three bytes of the code are used as the red, green, and blue values respectively.
    /// The highest byte is not taken into account, since the maximum possible value of a color hex
    /// code is `0xFFFFFF`.
    ///
    /// This function is mainly intended for creating `const` `RgbColor`s. For other cases,
    /// `RgbColor` implements `TryFrom<u32>`, which performs a check to ensure the value does not
    /// exceed the maximum possible value rather than silently discarding the highest byte.
    pub const fn hex(code: u32) -> Self {
        Self {
            r: ((code >> 16) & 0xFF) as u8,
            g: ((code >> 8) & 0xFF) as u8,
            b: (code & 0xFF) as u8,
        }
    }

    /// Encodes an RGB triple as a 32-bit number, where the last three bytes correspond to the red,
    /// green, and blue values respectively, and the highest byte is zeroed.
    pub const fn code(self) -> u32 {
        (self.r as u32) << 16 | (self.g as u32) << 8 | (self.b as u32)
    }

    /// Translates an 8-bit integer into an 8-bit color.
    pub const fn xterm(code: u8) -> Self {
        RgbColor::XTERM_256[code as usize]
    }

    /// Finds a color by its name in the standard list of [148 CSS colors]. Case-insensitive.
    ///
    /// [148 CSS colors]: https://www.w3.org/wiki/CSS/Properties/color/keywords
    pub fn named(name: &str) -> Option<RgbColor> {
        static LOOKUP: LazyLock<CaseFoldMap<&str, RgbColor>> = LazyLock::new(|| {
            NAMED_COLORS
                .iter()
                .map(|&(key, val)| (key.into(), val))
                .collect()
        });

        if name.starts_with('#') {
            return name.parse().ok();
        }
        LOOKUP.get(name).copied()
    }

    /// Iterates through colors in the standard list of [148 CSS colors].
    ///
    /// [148 CSS colors]: https://www.w3.org/wiki/CSS/Properties/color/keywords
    pub fn iter_named() -> NamedColorIter {
        NAMED_COLORS.iter().copied()
    }
}

impl fmt::Display for RgbColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(RgbDigits::prefixed(*self).as_str())
    }
}

impl fmt::UpperHex for RgbColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(RgbDigits::upper(*self).as_str())
    }
}

impl fmt::LowerHex for RgbColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(RgbDigits::lower(*self).as_str())
    }
}

impl TryFrom<u32> for RgbColor {
    type Error = HexOutOfRangeError;

    /// Parses a color from a color hex code. Fails if the value exceeds 0xFFFFFF.
    fn try_from(code: u32) -> Result<Self, Self::Error> {
        if code > 0xFFFFFF {
            Err(HexOutOfRangeError(code))
        } else {
            Ok(Self::hex(code))
        }
    }
}

impl From<RgbColor> for u32 {
    /// Encodes an RGB triple as a 32-bit number.
    fn from(value: RgbColor) -> Self {
        value.code()
    }
}

impl FromStr for RgbColor {
    type Err = ParseHexColorError;

    /// Parses a color from a color hex code string. The string must be a six-digit hexadecimal
    /// string prefixed by `#`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 7 || !s.starts_with('#') {
            return Err(ParseHexColorError::NotHex(s.to_owned()));
        }
        let code = u32::from_str_radix(&s[1..], 16)?;
        Ok(RgbColor::try_from(code)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_from_triple() {
        assert_eq!(RgbColor::rgb(1, 2, 3), RgbColor { r: 1, g: 2, b: 3 });
    }

    #[test]
    fn rgb_from_hex() {
        assert_eq!(RgbColor::hex(0x123456), RgbColor::rgb(0x12, 0x34, 0x56));
    }

    #[test]
    fn rgb_from_str() {
        assert_eq!("#123456".parse(), Ok(RgbColor::rgb(0x12, 0x34, 0x56)));
    }

    #[test]
    fn rgb_code() {
        assert_eq!(RgbColor::rgb(0x12, 0x34, 0x56).code(), 0x123456);
    }
}
