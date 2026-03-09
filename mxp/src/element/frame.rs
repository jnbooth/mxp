use std::borrow::Cow;
use std::str::FromStr;

use super::screen::Align;
use crate::Dimension;
use crate::argument::{Decoder, ExpectArg, Scan};
use crate::keyword::FrameKeyword;
use crate::parser::{Error, StringVariant, UnrecognizedVariant};

/// Action to apply to a [`Frame`].
///
/// See [MXP specification: `<FRAME>`](https://www.zuggsoft.com/zmud/mxp.htm#Frames).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum FrameAction {
    /// Create the specified window.
    #[default]
    Open,
    /// Close a window with the given name that was previously created.
    Close,
    /// Redirect all subsequent MUD output to the specified window.
    /// If an action of `REDIRECT` is used and the frame doesn't exist, it is created first as if an
    /// action of `OPEN` was specified, then output is redirected after the frame is created.
    Redirect,
}

impl StringVariant for FrameAction {
    type Variant = Self;
    const VARIANTS: &[Self] = &[Self::Open, Self::Close, Self::Redirect];
}

impl FromStr for FrameAction {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "open" => Ok(Self::Open),
            "close" => Ok(Self::Close),
            "redirect" => Ok(Self::Redirect),
            _ => Err(Self::Err::new(s)),
        }
    }
}

/// Alignment and position of a [`Frame`], which may either be an external (floating) frame or
/// an internal (docked) frame.
///
/// See [MXP specification: `<FRAME>`](https://www.zuggsoft.com/zmud/mxp.htm#Frames).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameLayout {
    /// Specifies that the frame is a floating frame.
    External {
        /// The coordinate of the left side of the frame. If a negative number is used, it means
        /// the value is relative to the right side of the screen instead of the left.
        left: Dimension<i32>,
        /// The coordinate of the top of the frame. If a negative value is used, it means the frame
        /// is relative to the bottom of the screen instead of the top.
        top: Dimension<i32>,
        /// The width of the frame.
        width: Option<Dimension<u32>>,
        /// The height of the frame.
        height: Option<Dimension<u32>>,
        /// Forces the frame to "stay on top" of the main MUD window.
        floating: bool,
    },
    /// Specifies that the frame is internal to the current MUD window.
    Internal {
        /// Specifies how the frame is docked with the MUD window.
        align: Align,
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

/// A separate browser window region with its own HTML document.
///
/// See [MXP specification: `<FRAME>`](https://www.zuggsoft.com/zmud/mxp.htm#Frames).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame<S = String> {
    /// The name of the frame to be used to send text to the frame later in the text stream. Several special names are recognized: `_top` specifies the main MUD window, `_previous` specifies the window that was active before this frame.
    pub name: S,
    /// Action to apply to the frame.
    pub action: FrameAction,
    /// Specifies the full caption of the frame. If undefined, `name` should be used as the title.
    pub title: Option<S>,
    /// Frame layout.
    pub layout: FrameLayout,
    /// Determines whether the frame is allowed to scroll.
    pub scrolling: bool,
}

impl<S> Frame<S> {
    /// `self.title`, or `self.name` if `self.title` is `None`.
    pub fn title(&self) -> &S {
        self.title.as_ref().unwrap_or(&self.name)
    }
}

impl Frame<&str> {
    pub fn into_owned(self) -> Frame<String> {
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
    pub fn into_owned(self) -> Frame<String> {
        Frame {
            name: self.name.into_owned(),
            action: self.action,
            title: self.title.map(Cow::into_owned),
            layout: self.layout,
            scrolling: self.scrolling,
        }
    }
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for Frame<Cow<'a, str>>
where
    D: Decoder,
    S: AsRef<str>,
{
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
            .is_some_and(|scrolling| scrolling.as_ref().eq_ignore_ascii_case("yes"));
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DestArgs<S> {
    pub name: S,
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for DestArgs<Cow<'a, str>>
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        Ok(Self {
            name: scanner.next()?.expect_some("name")?,
        })
    }
}
