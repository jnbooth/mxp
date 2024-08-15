use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use enumeration::Enum;

use crate::parser::UnrecognizedVariant;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Align {
    Left,
    Right,
    Top,
    Middle,
    Bottom,
}

impl Default for Align {
    fn default() -> Self {
        Self::Top
    }
}

impl FromStr for Align {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "left" => Self::Left,
            "right" => Self::Right,
            "top" => Self::Top,
            "middle" => Self::Middle,
            "bottom" => Self::Bottom,
            _ => return Err(Self::Err::new(s))
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum DimensionUnit {
    Pixel,
    CharacterHeight,
    Percentage,
}

impl Default for DimensionUnit {
    fn default() -> Self {
        Self::Pixel
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dimension<T = u32> {
    amount: T,
    unit: DimensionUnit,
}

impl<T> Dimension<T> {
    pub const fn pixels(amount: T) -> Self {
        Self {
            amount,
            unit: DimensionUnit::Pixel,
        }
    }

    pub const fn character_height(amount: T) -> Self {
        Self {
            amount,
            unit: DimensionUnit::CharacterHeight,
        }
    }

    pub const fn percentage(amount: T) -> Self {
        Self {
            amount,
            unit: DimensionUnit::Percentage,
        }
    }
}

impl<T: Display> Display for Dimension<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.amount.fmt(f)?;
        match self.unit {
            DimensionUnit::Pixel => Ok(()),
            DimensionUnit::CharacterHeight => f.write_str("c"),
            DimensionUnit::Percentage => f.write_str("%"),
        }
    }
}

impl<T: FromStr> FromStr for Dimension<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (unit, s) = match s.as_bytes().last() {
            Some(b'%') => (DimensionUnit::Percentage, &s[..s.len() - 1]),
            Some(b'c') => (DimensionUnit::CharacterHeight, &s[..s.len() - 1]),
            _ => (DimensionUnit::Pixel, s),
        };
        let amount = s.parse()?;
        Ok(Self { amount, unit })
    }
}
