#[cfg(feature = "serde")]
use serde::de::{Error as _, Unexpected};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;

use super::hex_color::HexColor;
use super::xterm::xterm;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorldColor {
    Ansi(u8),
    Hex(HexColor),
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
            Self::Hex(color) => write!(f, "Hex(#{color:X})"),
        }
    }
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

impl From<WorldColor> for HexColor {
    fn from(value: WorldColor) -> Self {
        match value {
            WorldColor::Ansi(code) => xterm(code),
            WorldColor::Hex(color) => color,
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for WorldColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            Self::Ansi(code) if serializer.is_human_readable() => {
                serializer.serialize_str(&code.to_string())
            }
            Self::Ansi(code) => serializer.serialize_u32(0xFFFFFF + 1 + code as u32),
            Self::Hex(color) => color.serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for WorldColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let code = <&str>::deserialize(deserializer)?;
            if code.starts_with('#') {
                match code.parse() {
                    Ok(hex) => Ok(Self::Hex(hex)),
                    Err(_) => Err(D::Error::invalid_value(
                        Unexpected::Str(code),
                        &"hex color code or stringified integer between 0 and 255",
                    )),
                }
            } else {
                match code.parse() {
                    Ok(ansi) => Ok(Self::Ansi(ansi)),
                    Err(_) => Err(D::Error::invalid_value(
                        Unexpected::Str(code),
                        &"hex color code or stringified integer between 0 and 255",
                    )),
                }
            }
        } else {
            let code = u32::deserialize(deserializer)?;
            if code <= 0xFFFFFF {
                Ok(Self::Hex(HexColor::new(code)))
            } else if code <= 0xFFFFFF + 16 {
                Ok(Self::Ansi((code - 0xFFFFFF - 1) as u8))
            } else {
                Err(D::Error::invalid_value(
                    Unexpected::Unsigned(code as u64),
                    &"integer between 0x000000 and 0x100000F",
                ))
            }
        }
    }
}
