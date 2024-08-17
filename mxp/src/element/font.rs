use crate::argument::{Decoder, Scan};
use crate::color::RgbColor;
use crate::parser::Error;
use crate::parser::UnrecognizedVariant;
use enumeration::Enum;
use std::borrow::Cow;
use std::num::NonZeroU8;
use std::str;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum FontStyle {
    Blink,
    Bold,
    Inverse,
    Italic,
    Underline,
}

impl FromStr for FontStyle {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "blink" => Self::Blink,
            "bold" => Self::Bold,
            "inverse" => Self::Inverse,
            "italic" => Self::Italic,
            "underline" => Self::Underline,
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
    pub fn into_owned(&self) -> Font {
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

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for Font<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
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
