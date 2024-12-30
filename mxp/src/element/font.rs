use crate::argument::{Decoder, Scan};
use crate::color::RgbColor;
use crate::parser::Error;
use crate::parser::UnrecognizedVariant;
use flagset::flags;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::num::NonZeroU8;
use std::str;
use std::str::FromStr;

flags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialOrd, Ord, Hash)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontEffect {
    Color(RgbColor),
    Style(FontStyle),
}

impl FontEffect {
    pub fn parse(s: &str) -> Option<Self> {
        match s.parse() {
            Ok(style) => Some(Self::Style(style)),
            Err(_) => RgbColor::named(s).map(Self::Color),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FgColor<S> {
    pub(crate) inner: S,
}

impl<S: AsRef<str>> FgColor<S> {
    pub fn iter(&self) -> impl Iterator<Item = FontEffect> + '_ {
        self.inner.as_ref().split(',').filter_map(FontEffect::parse)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Font<S = String> {
    pub face: Option<S>,
    pub size: Option<NonZeroU8>,
    pub color: Option<FgColor<S>>,
    pub back: Option<RgbColor>,
}

impl<'a> Font<&'a str> {
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

impl<'a> Font<Cow<'a, str>> {
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

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Font<Cow<'a, str>> {
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
