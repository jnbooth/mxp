use crate::color::RgbColor;
use enumeration::Enum;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum FontStyle {
    Blink,
    Bold,
    Inverse,
    Italic,
    Underline,
}

impl FontStyle {
    fn parse(s: &str) -> Option<Self> {
        match_ci! {s,
            "blink" => Some(FontStyle::Blink),
            "bold" => Some(FontStyle::Bold),
            "inverse" => Some(FontStyle::Inverse),
            "italic" => Some(FontStyle::Italic),
            "underline" => Some(FontStyle::Underline),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontEffect {
    Color(RgbColor),
    Style(FontStyle),
}

impl FontEffect {
    pub fn parse(s: &str) -> Option<Self> {
        match FontStyle::parse(s) {
            Some(style) => Some(Self::Style(style)),
            None => RgbColor::named(s).map(Self::Color),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FgColor<S> {
    pub(crate) inner: S,
}

impl<S: AsRef<str>> FgColor<S> {
    pub fn iter(&self) -> impl Iterator<Item = FontEffect> + '_ {
        self.inner.as_ref().split(',').flat_map(FontEffect::parse)
    }
}
