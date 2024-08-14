use std::str::FromStr;

use crate::color::RgbColor;
use crate::parser::UnrecognizedVariant;
use enumeration::Enum;

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
