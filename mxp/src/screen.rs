use std::fmt;
use std::str::FromStr;

use crate::parse::UnrecognizedVariant;

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

impl_parse_enum!(Align, Top, Bottom, Left, Right, Middle);

impl fmt::Display for Align {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

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

impl fmt::Display for DimensionUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Pixel => Ok(()),
            Self::Percentage => "%".fmt(f),
            Self::CharacterSpacing => "c".fmt(f),
        }
    }
}

/// A measurement of screen space, specified as a pixel amount, a percentage, or an amount of
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
        let Self { amount, unit } = self;
        write!(f, "{amount}{unit}")
    }
}

impl<T: FromStr> FromStr for Dimension<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (amount, unit) = match s.split_at_checked(s.len().saturating_sub(1)) {
            Some((amount, "%")) => (amount, DimensionUnit::Percentage),
            Some((amount, "c")) => (amount, DimensionUnit::CharacterSpacing),
            _ => (s, DimensionUnit::Pixel),
        };
        Ok(Self {
            amount: amount.parse()?,
            unit,
        })
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
