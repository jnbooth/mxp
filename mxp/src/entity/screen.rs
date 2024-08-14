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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dimension {
    amount: u32,
    unit: DimensionUnit,
}

impl FromStr for Dimension {
    type Err = <u32 as FromStr>::Err;

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
