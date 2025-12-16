use std::fmt;
use std::str::FromStr;

use crate::parser::UnrecognizedVariant;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub amount: T,
    pub unit: DimensionUnit,
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

impl<T: fmt::Display> fmt::Display for Dimension<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.unit {
            DimensionUnit::Pixel => write!(f, "{}", self.amount),
            DimensionUnit::CharacterHeight => write!(f, "{}c", self.amount),
            DimensionUnit::Percentage => write!(f, "{}%", self.amount),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{StringPair, format_from_pairs, parse_from_pairs};

    const DIMENSION_PAIRS: &[StringPair<Dimension>] = &[
        (Dimension::pixels(10), "10"),
        (Dimension::character_height(20), "20c"),
        (Dimension::percentage(30), "30%"),
    ];

    #[test]
    fn fmt_dimension() {
        let (actual, expected) = format_from_pairs(DIMENSION_PAIRS);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_dimension() {
        let (actual, expected) = parse_from_pairs(DIMENSION_PAIRS);
        assert_eq!(actual, expected);
    }
}
