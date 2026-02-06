use std::borrow::Cow;
use std::iter::FilterMap;
use std::num::NonZero;
use std::str::FromStr;
use std::str::{self, Split};

use flagset::flags;

use crate::argument::{Decoder, Scan};
use crate::color::RgbColor;
use crate::parser::Error;
use crate::parser::UnrecognizedVariant;

flags! {
    pub enum FontStyle: u8 {
        Blink,
        Bold,
        Italic,
        Underline,
        Inverse,
    }
}

impl FromStr for FontStyle {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "blink" => Self::Blink,
            "bold" => Self::Bold,
            "italic" => Self::Italic,
            "underline" => Self::Underline,
            "inverse" => Self::Inverse,
            _ => return Err(Self::Err::new(s)),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FontEffect {
    Color(RgbColor),
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Font<S = String> {
    pub face: Option<S>,
    pub size: Option<NonZero<u8>>,
    pub color: Option<FgColor<S>>,
    pub back: Option<RgbColor>,
}

impl Font<&str> {
    pub fn into_owned(self) -> Font {
        Font {
            face: self.face.map(ToOwned::to_owned),
            size: self.size,
            color: self.color.map(|color| FgColor {
                inner: color.inner.to_owned(),
            }),
            back: self.back,
        }
    }
}

impl Font<Cow<'_, str>> {
    pub fn into_owned(self) -> Font {
        Font {
            face: self.face.map(Cow::into_owned),
            size: self.size,
            color: self.color.map(|color| FgColor {
                inner: color.inner.into_owned(),
            }),
            back: self.back,
        }
    }
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for Font<Cow<'a, str>>
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        Ok(Self {
            face: scanner.next_or("face")?,
            size: scanner
                .next_or("size")?
                .and_then(|size| size.as_ref().parse().ok()),
            color: scanner
                .next_or("color")?
                .map(|color| FgColor { inner: color }),
            back: scanner
                .next_or("back")?
                .and_then(|back| RgbColor::named(back.as_ref())),
        })
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
