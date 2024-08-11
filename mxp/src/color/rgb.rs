use super::error::{HexOutOfRangeError, ParseHexColorError};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const BLACK: Self = Self::hex(0x000000);
    pub const WHITE: Self = Self::hex(0xFFFFFF);

    /// Constructs an `RgbColor` from a red, green, and blue value.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Constructs an `RgbColor` from a color hex code.
    /// The last three bytes of the code are used as the red, green, and blue values respectively.
    /// Higher bytes are not taken into account, since the maximum possible value of a color hex
    /// code is `0xFFFFFF`.
    pub const fn hex(code: u32) -> Self {
        Self {
            r: ((code >> 16) & 0xFF) as u8,
            g: ((code >> 8) & 0xFF) as u8,
            b: (code & 0xFF) as u8,
        }
    }

    /// Encodes an RGB triple as a 32-bit number, where the last three bytes correspond to the red,
    /// green, and blue values respectively.
    /// Higher bytes are zeroed, since the maximum possible value of a color hex code is `0xFFFFFF`.
    pub const fn code(self) -> u32 {
        (self.r as u32) << 16 | (self.g as u32) << 8 | (self.b as u32)
    }
}

impl Display for RgbColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#{:0>6X}", self.code())
    }
}

impl fmt::UpperHex for RgbColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:0>6X}", self.code())
    }
}

impl fmt::LowerHex for RgbColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:0>6x}", self.code())
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
        if !s.starts_with('#') {
            return Err(ParseHexColorError::NotHex(s.to_owned()));
        }
        let code = u32::from_str_radix(&s[1..], 16)?;
        if code > 0xFFFFFF {
            return Err(ParseHexColorError::OutOfRange(code));
        }
        Ok(RgbColor::hex(code))
    }
}
