use std::str::FromStr;

use crate::color::RgbColor;
use crate::parse::{Decoder, Error, ExpectArg as _, Scan};

/// Sets the color of the text.
///
/// See [MXP specification: `<COLOR>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Color {
    /// If defined, sets the foreground color of the text.
    pub fore: Option<RgbColor>,
    /// If defined, sets the background color of the text.
    pub back: Option<RgbColor>,
}

impl<'a, D> TryFrom<Scan<'a, D>> for Color
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        let fore = scanner.next_or("fore")?.expect_color()?;
        let back = scanner.next_or("back")?.expect_color()?;
        scanner.expect_end()?;
        Ok(Self { fore, back })
    }
}

impl FromStr for Color {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Color)
    }
}
