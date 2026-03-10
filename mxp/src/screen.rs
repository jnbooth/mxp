use std::fmt;
use std::str::FromStr;

use crate::parser::{StringVariant, UnrecognizedVariant};

/// Alignment of an on-screen item.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
    Middle,
}

impl StringVariant for Align {
    type Variant = Self;
    const VARIANTS: &[Self] = &[
        Self::Top,
        Self::Bottom,
        Self::Left,
        Self::Right,
        Self::Middle,
    ];
}

impl_parse_enum!(Align, Top, Bottom, Left, Right, Middle);

/// Specifies the units of a [`Dimension`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum DimensionUnit {
    /// The amount is measured in pixels. For example, a frame with `LEFT=50 TOP=25` starts 50
    /// pixels from the left edge of the screen and 25 pixels from the top edge of the screen.
    #[default]
    Pixel,
    /// The amount is measured as a percentage. For example, a frame with `LEFT="50%" TOP="25%"`
    /// starts halfway across the screen and 25% of the way from the top to the bottom.
    Percentage = b'%' as _,
    /// The amount is measured in character size. For example, a frame with `LEFT="50c" TOP="25c"`
    /// starts 50 character spacings from the left side of the screen (using the width of the
    /// character X if it is a proportional font), and 25 character spacings from the top of the
    /// screen (using the height of the capital X character).
    CharacterSpacing = b'c' as _,
}

/// A measurement of screen space, specified as a pixel amount, a perentage, or an amount of
/// character widths, as determined by the [`DimensionUnit`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Dimension<T = u32> {
    pub amount: T,
    pub unit: DimensionUnit,
}

impl<T> Dimension<T> {
    /// Constructs a `Dimension` for the specified amount in pixels ([`DimensionUnit::Pixel`]).
    pub const fn pixels(amount: T) -> Self {
        Self {
            amount,
            unit: DimensionUnit::Pixel,
        }
    }

    /// Constructs a `Dimension` for the specified amount in character spacing
    /// ([`DimensionUnit::CharacterSpacing`]).
    pub const fn character_spacing(amount: T) -> Self {
        Self {
            amount,
            unit: DimensionUnit::CharacterSpacing,
        }
    }

    /// Constructs a `Dimension` for the specified amount in screen percentage
    /// ([`DimensionUnit::Percentage`]).
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
            DimensionUnit::CharacterSpacing => write!(f, "{}c", self.amount),
            DimensionUnit::Percentage => write!(f, "{}%", self.amount),
        }
    }
}

impl<T: FromStr> FromStr for Dimension<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (unit, s) = match s.as_bytes().last() {
            Some(b'%') => (DimensionUnit::Percentage, &s[..s.len() - 1]),
            Some(b'c') => (DimensionUnit::CharacterSpacing, &s[..s.len() - 1]),
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
        (Dimension::character_spacing(20), "20c"),
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
