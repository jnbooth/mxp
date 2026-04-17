use std::fmt;
use std::str::FromStr;

use crate::arguments::{ArgumentScanner, ExpectArg, FromArgs};
use crate::color::RgbColor;

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

impl<'a, S: AsRef<str>> FromArgs<'a, S> for Color {
    fn from_args<A: ArgumentScanner<'a, Decoded = S>>(mut scanner: A) -> crate::Result<Self> {
        let fore = scanner.get_next_or("fore")?.expect_color()?;
        let back = scanner.get_next_or("back")?.expect_color()?;
        scanner.expect_end()?;
        Ok(Self { fore, back })
    }
}

impl From<RgbColor> for Color {
    /// Constructs a `<COLOR>` with the specified color as its foreground, and no background color.
    fn from(fore: RgbColor) -> Self {
        Self {
            fore: Some(fore),
            ..Default::default()
        }
    }
}

impl<'a> TryFrom<&'a str> for Color {
    type Error = crate::parse::FromStrError;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        crate::parse::parse_element(s)
    }
}
impl FromStr for Color {
    type Err = crate::parse::FromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element::<Color>(s)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Color { fore, back } = self;
        crate::display::ElementFormatter {
            name: "COLOR",
            arguments: &[fore, back],
            keywords: &[],
        }
        .fmt(f)
    }
}
