use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::num::ParseIntError;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HexOutOfRangeError(pub u32);

impl Display for HexOutOfRangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("number exceeds maximum hex code value")
    }
}

impl Error for HexOutOfRangeError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseHexColorError {
    NotHex(String),
    NotU32(ParseIntError),
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

impl Display for ParseHexColorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotHex(_) => f.write_str("value is not formatted as a hex code"),
            Self::NotU32(error) => error.fmt(f),
            Self::OutOfRange(_) => f.write_str("number exceeds maximum hex code value"),
        }
    }
}
