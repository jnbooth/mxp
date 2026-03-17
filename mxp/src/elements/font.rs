use std::borrow::Cow;
use std::fmt;
use std::num::NonZero;

use flagset::{FlagSet, flags};

use crate::arguments::{ArgumentScanner, ExpectArg as _};
use crate::color::RgbColor;
use crate::parse::{Decoder, UnrecognizedVariant};

flags! {
    /// Font modifier applied by the [`color`](Font::color) argument of a [`Font`] tag.
    pub enum FontStyle: u8 {
        Blink,
        Bold,
        Italic,
        Underline,
        Inverse,
    }
}

impl_parse_enum!(FontStyle, Blink, Bold, Italic, Underline, Inverse);

impl fmt::Display for FontStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// Changes the font for subsequent text.
///
/// See [MXP specification: `<FONT>`](https://www.zuggsoft.com/zmud/mxp.htm#MXP%20Tags).
///
/// # Examples
///
/// ```
/// use mxp::RgbColor;
///
/// assert_eq!(
///     "<FONT 'Times New Roman' SIZE=10 COLOR=black,blink BACK=#123456>".parse::<mxp::Font>(),
///     Ok(mxp::Font {
///         face: Some("Times New Roman".into()),
///         size: 10.try_into().ok(),
///         color: Some(RgbColor::hex(0x000000)),
///         back: Some(RgbColor::hex(0x123456)),
///         style: mxp::FontStyle::Blink.into(),
///     }),
/// );
/// ```
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

impl<S: AsRef<str>> Font<S> {
    pub(crate) fn scan<A>(mut scanner: A) -> crate::Result<Self>
    where
        A: ArgumentScanner<Output = S>,
    {
        let face = scanner.decode_next_or("face")?;
        let size = scanner
            .decode_next_or("size")?
            .expect_number()?
            .and_then(NonZero::new);
        let fore = scanner.decode_next_or("color")?;
        let back = scanner.decode_next_or("back")?.expect_color()?;
        let mut color: Option<RgbColor> = None;
        let mut style: FlagSet<FontStyle> = FlagSet::empty();
        if let Some(fore) = fore {
            for effect in fore.as_ref().split(',') {
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

impl_from_str!(Font);

impl<S: AsRef<str>> fmt::Display for Font<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Font {
            face,
            size,
            color,
            style,
            back,
        } = self.borrow_text();
        crate::display::ElementFormatter {
            name: "FONT",
            arguments: &[&face, &size, &(color, style), &back],
            keywords: &[],
        }
        .fmt(f)
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
