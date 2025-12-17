use std::borrow::Cow;
use std::str::FromStr;

use super::screen::Align;
use crate::Dimension;
use crate::argument::{Decoder, ExpectArg, Scan};
use crate::keyword::FrameKeyword;
use crate::parser::{Error, UnrecognizedVariant};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum FrameAction {
    #[default]
    Open,
    Close,
    Redirect,
}

impl FromStr for FrameAction {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "OPEN" => Self::Open,
            "CLOSE" => Self::Close,
            "REDIRECT" => Self::Redirect,
            _ => return Err(Self::Err::new(s)),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrameLayout {
    Internal {
        align: Align,
    },
    External {
        left: Dimension<i32>,
        top: Dimension<i32>,
        width: Option<Dimension<u32>>,
        height: Option<Dimension<u32>>,
        floating: bool,
    },
}

impl Default for FrameLayout {
    fn default() -> Self {
        Self::External {
            left: Dimension::pixels(0),
            top: Dimension::pixels(0),
            width: None,
            height: None,
            floating: false,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Frame<S = String> {
    pub name: S,
    pub action: FrameAction,
    pub title: Option<S>,
    pub layout: FrameLayout,
    pub scrolling: bool,
}

impl Frame<&str> {
    pub fn into_owned(self) -> Frame {
        Frame {
            name: self.name.to_owned(),
            action: self.action,
            title: self.title.map(ToOwned::to_owned),
            layout: self.layout,
            scrolling: self.scrolling,
        }
    }
}

impl Frame<Cow<'_, str>> {
    pub fn into_owned(self) -> Frame {
        Frame {
            name: self.name.into_owned(),
            action: self.action,
            title: self.title.map(Cow::into_owned),
            layout: self.layout,
            scrolling: self.scrolling,
        }
    }
}

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Frame<Cow<'a, str>> {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        let name = scanner.next_or("name")?.expect_some("name")?;
        let action = scanner
            .next_or("action")?
            .and_then(|action| action.as_ref().parse().ok())
            .unwrap_or_default();
        let title = scanner.next_or("title")?;
        let align: Align = scanner
            .next_or("align")?
            .and_then(|align| align.as_ref().parse().ok())
            .unwrap_or_default();
        let left = scanner
            .next_or("left")?
            .expect_number()?
            .unwrap_or_default();
        let top = scanner.next_or("top")?.expect_number()?.unwrap_or_default();
        let width = scanner.next_or("width")?.expect_number()?;
        let height = scanner.next_or("height")?.expect_number()?;
        let scrolling = scanner
            .next_or("scrolling")?
            .is_some_and(|scrolling| scrolling.as_ref().eq_ignore_ascii_case("YES"));
        let keywords = scanner.into_keywords();
        let layout = if keywords.contains(FrameKeyword::Internal) {
            FrameLayout::Internal { align }
        } else {
            FrameLayout::External {
                left,
                top,
                width,
                height,
                floating: keywords.contains(FrameKeyword::Floating),
            }
        };
        Ok(Self {
            name,
            action,
            title,
            layout,
            scrolling,
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DestArgs<S> {
    pub name: S,
}

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for DestArgs<Cow<'a, str>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        Ok(Self {
            name: scanner.next()?.expect_some("name")?,
        })
    }
}
