use std::borrow::Cow;
use std::num::NonZero;
use std::str::{self, FromStr};

use flagset::{FlagSet, flags};

use crate::color::RgbColor;
use crate::parse::{Decoder, Error, ExpectArg as _, Scan, UnrecognizedVariant};

flags! {
    /// Font modifier applied by the [`color`] argument of a [`Font`] tag.
    pub enum FontStyle: u8 {
        Blink,
        Bold,
        Italic,
        Underline,
        Inverse,
    }
}

impl_parse_enum!(FontStyle, Blink, Bold, Italic, Underline, Inverse);

/// Changes the font for subsequent text.
///
/// See [MXP specification: `<FONT>`](https://www.zuggsoft.com/zmud/mxp.htm#MXP%20Tags).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Font<S = String> {
    /// Font family.
    pub face: Option<S>,
    /// Font size.
    pub size: Option<NonZero<u8>>,
    /// Foreground color.
    pub color: Option<RgbColor>,
    /// Style effects. These are parsed from the `color` argument.
    pub style: FlagSet<FontStyle>,
    /// Background color.
    pub back: Option<RgbColor>,
}

impl<S> Font<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, mut f: F) -> Font<T>
    where
        F: FnMut(S) -> T,
    {
        Font {
            face: self.face.map(&mut f),
            size: self.size,
            color: self.color,
            style: self.style,
            back: self.back,
        }
    }
}

impl_into_owned!(Font);

impl<S: AsRef<str>> Font<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Font<&str> {
        Font {
            face: self.face.as_ref().map(AsRef::as_ref),
            size: self.size,
            color: self.color,
            style: self.style,
            back: self.back,
        }
    }
}

impl_partial_eq!(Font);

impl<'a, D> TryFrom<Scan<'a, D>> for Font<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        let face = scanner.next_or("face")?;
        let size = scanner
            .next_or("size")?
            .expect_number()?
            .and_then(NonZero::new);
        let fore = scanner.next_or("color")?;
        let back = scanner.next_or("back")?.expect_color()?;
        let mut color: Option<RgbColor> = None;
        let mut style: FlagSet<FontStyle> = FlagSet::empty();
        if let Some(fore) = fore {
            for effect in fore.split(',') {
                let effect = effect.trim_ascii();
                if let Ok(flag) = effect.parse::<FontStyle>() {
                    style |= flag;
                } else if let Some(rgb) = RgbColor::named(effect) {
                    color = Some(rgb);
                }
            }
        }
        scanner.expect_end()?;
        Ok(Self {
            face,
            size,
            color,
            style,
            back,
        })
    }
}

impl FromStr for Font {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Font)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fg_color() {
        let el = "<font color=\"a,bold,b,blink,black,c\">"
            .parse::<Font>()
            .expect("parse error");
        assert_eq!(el.color, Some(RgbColor::BLACK));
        assert_eq!(el.style, FontStyle::Blink | FontStyle::Bold);
    }
}
