use std::fmt;
use std::str::{self, FromStr};

use super::error::{HexOutOfRangeError, ParseHexColorError};
use super::named::{NAMED_COLORS, NamedColorIter, get_named_color};
use super::xterm::{XTERM_COLORS, first_xterm_colors};

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
    #[inline]
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
    #[inline]
    pub const fn hex(code: u32) -> Self {
        debug_assert!(code <= 0xFFFFFF);

        #[cfg(target_endian = "big")]
        let [_, r, g, b] = code.to_ne_bytes();
        #[cfg(target_endian = "little")]
        let [b, g, r, _] = code.to_ne_bytes();
        Self { r, g, b }
    }

    /// Encodes an RGB triple as a 32-bit number, where the last three bytes correspond to the red,
    /// green, and blue values respectively, and the highest byte is zeroed.
    #[inline]
    pub const fn code(self) -> u32 {
        #[cfg(target_endian = "big")]
        let bytes = [0, self.r, self.g, self.b];
        #[cfg(target_endian = "little")]
        let bytes = [self.b, self.g, self.r, 0];
        u32::from_ne_bytes(bytes)
    }

    /// Translates an 8-bit integer into an 8-bit color.
    #[inline]
    pub const fn xterm(code: u8) -> Self {
        RgbColor::XTERM_256[code as usize]
    }

    pub const fn named(name: &str) -> Option<RgbColor> {
        const fn hex_digit(byte: u8) -> u8 {
            if byte > b'9' {
                const TO_UPPERCASE_MASK: u8 = !0b0010_0000;
                (byte.wrapping_sub(b'A') & TO_UPPERCASE_MASK) + 10
            } else {
                byte.wrapping_sub(b'0')
            }
        }

        const MAX_COLOR_LEN: usize = {
            let mut max_len = 0;
            let mut i = 0;
            while i < NAMED_COLORS.len() {
                let len = NAMED_COLORS[i].0.len();
                if len > max_len {
                    max_len = len;
                }
                i += 1;
            }
            max_len
        };

        if let &[b'#', r0, r1, g0, g1, b0, b1] = name.as_bytes() {
            let (r0, r1, g0, g1, b0, b1) = (
                hex_digit(r0),
                hex_digit(r1),
                hex_digit(g0),
                hex_digit(g1),
                hex_digit(b0),
                hex_digit(b1),
            );
            if r0 >= 16 || r1 >= 16 || g0 >= 16 || g1 >= 16 || b0 >= 16 || b1 >= 16 {
                return None;
            }
            return Some(Self {
                r: r0 << 4 | r1,
                g: g0 << 4 | g1,
                b: b0 << 4 | b1,
            });
        }

        let mut buf = [0; MAX_COLOR_LEN];

        let Some((name_lower, _)) = buf.split_at_mut_checked(name.len()) else {
            return None;
        };

        name_lower.copy_from_slice(name.as_bytes());
        name_lower.make_ascii_lowercase();
        get_named_color(name_lower)
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
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl fmt::UpperHex for RgbColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl fmt::LowerHex for RgbColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02x}{:02x}{:02x}", self.r, self.g, self.b)
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
    fn rgb_from_name() {
        assert_eq!(
            RgbColor::named("DARKblue"),
            Some(RgbColor::rgb(0x00, 0x00, 0x8B))
        );
    }

    #[test]
    fn rgb_from_hex_code() {
        assert_eq!(
            RgbColor::named("#F23456"),
            Some(RgbColor::rgb(0xF2, 0x34, 0x56))
        );
    }

    #[test]
    fn rgb_code() {
        assert_eq!(RgbColor::rgb(0x12, 0x34, 0x56).code(), 0x123456);
    }
}
