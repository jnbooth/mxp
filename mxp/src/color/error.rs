use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

/// An error which is returned when attempting to create an [`RgbColor`](crate::RgbColor) from a
/// number greater than the maximum hex code value (`0xFFFFFF`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct HexOutOfRangeError(pub u32);

impl fmt::Display for HexOutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("number exceeds maximum hex code value")
    }
}

impl Error for HexOutOfRangeError {}

/// An error which can be returned when parsing an [`RgbColor`](crate::RgbColor).
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseHexColorError {
    /// The string is not in hexadecimal format (`"#ABCDEF"`).
    NotHex(String),
    /// The string could not be parsed to an integer.
    NotU32(ParseIntError),
    /// The integer parsed from the string exceeds the maximum hex code value (`0xFFFFFF`).
    OutOfRange(u32),
}

impl From<HexOutOfRangeError> for ParseHexColorError {
    fn from(value: HexOutOfRangeError) -> Self {
        Self::OutOfRange(value.0)
    }
}

impl From<ParseIntError> for ParseHexColorError {
    fn from(value: ParseIntError) -> Self {
        Self::NotU32(value)
    }
}

impl fmt::Display for ParseHexColorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotHex(_) => f.write_str("value is not formatted as a hex code"),
            Self::NotU32(error) => error.fmt(f),
            Self::OutOfRange(_) => f.write_str("number exceeds maximum hex code value"),
        }
    }
}
