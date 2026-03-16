use std::str::FromStr;

use crate::arguments::{ArgumentScanner, ExpectArg as _};
use crate::color::RgbColor;
use crate::parse::{Decoder, Scan};

/// Sets the color of the text.
///
/// See [MXP specification: `<COLOR>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting).
///
/// # Examples
///
/// ```
/// use mxp::RgbColor;
///
/// assert_eq!(
///     "<COLOR FORE=red BACK=#123456>".parse::<mxp::Color>(),
///     Ok(mxp::Color {
///         fore: Some(RgbColor::hex(0xFF0000)),
///         back: Some(RgbColor::hex(0x123456)),
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Color {
    /// If defined, sets the foreground color of the text.
    pub fore: Option<RgbColor>,
    /// If defined, sets the background color of the text.
    pub back: Option<RgbColor>,
}

impl Color {
    pub(crate) fn scan<A>(mut scanner: A) -> crate::Result<Self>
    where
        A: ArgumentScanner,
    {
        let fore = scanner.next_or("fore")?.expect_color()?;
        let back = scanner.next_or("back")?.expect_color()?;
        scanner.expect_end()?;
        Ok(Self { fore, back })
    }
}

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Color {
    type Error = crate::Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        Self::scan(scanner)
    }
}

impl FromStr for Color {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Color)
    }
}
