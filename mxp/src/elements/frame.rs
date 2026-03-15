use std::borrow::Cow;
use std::str::FromStr;

use crate::keyword::FrameKeyword;
use crate::parse::{Decoder, ExpectArg as _, Scan, UnrecognizedVariant};
use crate::screen::{Align, Dimension};

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

impl_parse_enum!(FrameAction, Open, Close, Redirect);

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
///
/// # Examples
///
/// ```
/// use mxp::{Align, Dimension, FrameAction, FrameLayout};
///
/// assert_eq!(
///     "<FRAME NAME=Map Left=-20c Top=0 Width=20c Height=20c>".parse::<mxp::Frame>(),
///     Ok(mxp::Frame {
///         name: "Map".into(),
///         action: FrameAction::Open,
///         title: "Map".into(),
///         scrolling: false,
///         layout: FrameLayout::External {
///             left: Dimension::character_spacing(-20),
///             top: Dimension::pixels(0),
///             width: Some(Dimension::character_spacing(20)),
///             height: Some(Dimension::character_spacing(20)),
///             floating: false,
///         },
///     }),
/// );
///
/// assert_eq!(
///     "<FRAME NAME=Tells REDIRECT INTERNAL ALIGN=top SCROLLING=yes>".parse::<mxp::Frame>(),
///     Ok(mxp::Frame {
///         name: "Tells".into(),
///         action: FrameAction::Redirect,
///         title: "Tells".into(),
///         scrolling: true,
///         layout: FrameLayout::Internal {
///             align: Align::Top,
///         },
///     }),
/// );
///
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame<S = String> {
    /// The name of the frame to be used to send text to the frame later in the text stream. Several special names are recognized: `_top` specifies the main MUD window, `_previous` specifies the window that was active before this frame.
    pub name: S,
    /// Action to apply to the frame.
    pub action: FrameAction,
    /// Specifies the full caption of the frame.
    pub title: S,
    /// Frame layout.
    pub layout: FrameLayout,
    /// Determines whether the frame is allowed to scroll.
    pub scrolling: bool,
}

impl<S> Frame<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, mut f: F) -> Frame<T>
    where
        F: FnMut(S) -> T,
    {
        Frame {
            name: f(self.name),
            action: self.action,
            title: f(self.title),
            layout: self.layout,
            scrolling: self.scrolling,
        }
    }
}

impl_into_owned!(Frame);

impl<S: AsRef<str>> Frame<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Frame<&str> {
        Frame {
            name: self.name.as_ref(),
            action: self.action,
            title: self.title.as_ref(),
            layout: self.layout,
            scrolling: self.scrolling,
        }
    }
}

impl_partial_eq!(Frame);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum YesOrNo {
    No,
    Yes,
}

impl_parse_enum!(YesOrNo, No, Yes);

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Frame<Cow<'a, str>> {
    type Error = crate::Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        let name = scanner.next_or("name")?.expect_some("name")?;
        let action = scanner
            .next_or("action")?
            .expect_variant()?
            .unwrap_or_default();
        let title = scanner.next_or("title")?.unwrap_or(name.clone());
        let align = scanner
            .next_or("align")?
            .expect_variant()?
            .unwrap_or_default();
        let left = scanner
            .next_or("left")?
            .expect_number()?
            .unwrap_or_default();
        let top = scanner.next_or("top")?.expect_number()?.unwrap_or_default();
        let width = scanner.next_or("width")?.expect_number()?;
        let height = scanner.next_or("height")?.expect_number()?;
        let scrolling = scanner.next_or("scrolling")?.expect_variant()? == Some(YesOrNo::Yes);
        let keywords = scanner.into_keywords()?;
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

impl FromStr for Frame {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Frame)
    }
}
