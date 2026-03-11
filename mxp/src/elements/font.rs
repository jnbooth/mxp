use std::borrow::Cow;
use std::iter::FilterMap;
use std::num::NonZero;
use std::str::{self, FromStr, Split};

use flagset::flags;

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

/// Font effect applied by the [`color`] argument of a [`Font`] tag.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FontEffect {
    /// Change foreground color.
    Color(RgbColor),
    /// Change text style.
    Style(FontStyle),
}

impl FontEffect {
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s.parse() {
            Ok(style) => Some(Self::Style(style)),
            Err(_) => RgbColor::named(s).map(Self::Color),
        }
    }
}

/// [`color`] argument of a font tag, e.g. `<FONT color=red,bold,italic>`.
///
/// `FgColor` is an iterator over [`FontEffect`]s.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct FgColor<S> {
    pub(crate) inner: S,
}

impl<S: AsRef<str>> FgColor<S> {
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl<'a, S: AsRef<str>> IntoIterator for &'a FgColor<S> {
    type Item = FontEffect;

    type IntoIter = FilterMap<Split<'a, char>, fn(&str) -> Option<FontEffect>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.as_ref().split(',').filter_map(FontEffect::parse)
    }
}

/// Changes the font for subsequent text.
///
/// See [MXP specification: `<FONT>`](https://www.zuggsoft.com/zmud/mxp.htm#MXP%20Tags).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Font<S = String> {
    /// Font family.
    pub face: Option<S>,
    /// Font size.
    pub size: Option<NonZero<u8>>,
    /// Foreground color and style effects.
    pub color: Option<FgColor<S>>,
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
            color: self.color.map(|color| FgColor {
                inner: f(color.inner),
            }),
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
            color: self.color.as_ref().map(|color| FgColor {
                inner: color.inner.as_ref(),
            }),
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
        Ok(Self {
            face: scanner.next_or("face")?,
            size: scanner.next_or("size")?.and_then(|size| size.parse().ok()),
            color: scanner
                .next_or("color")?
                .map(|color| FgColor { inner: color }),
            back: scanner.next_or("back")?.color(),
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
        let fg = FgColor {
            inner: "a,bold,b,black,c",
        };
        assert_eq!(
            fg.iter().collect::<Vec<_>>(),
            vec![
                FontEffect::Style(FontStyle::Bold),
                FontEffect::Color(RgbColor::BLACK)
            ]
        );
    }
}
