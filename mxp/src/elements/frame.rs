use std::borrow::Cow;
use std::fmt;

use crate::arguments::{ArgumentScanner, ExpectArg as _};
use crate::keyword::FrameKeyword;
use crate::parse::{Decoder, UnrecognizedVariant};
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
impl_display_enum!(FrameAction, Open, Close, Redirect);

/// Alignment of an on-screen item.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum FrameAlign {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
    Middle,
    /// Indicates that the frame should be grouped into a tab with its parent (docked) window.
    Client,
}

impl_parse_enum!(FrameAlign, Top, Bottom, Left, Right, Middle, Client);

impl fmt::Display for FrameAlign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<Align> for FrameAlign {
    fn from(value: Align) -> Self {
        match value {
            Align::Top => Self::Top,
            Align::Bottom => Self::Bottom,
            Align::Left => Self::Left,
            Align::Right => Self::Right,
            Align::Middle => Self::Middle,
        }
    }
}

impl TryFrom<FrameAlign> for Align {
    type Error = UnrecognizedVariant<Self>;

    fn try_from(value: FrameAlign) -> Result<Self, Self::Error> {
        match value {
            FrameAlign::Top => Ok(Self::Top),
            FrameAlign::Bottom => Ok(Self::Bottom),
            FrameAlign::Left => Ok(Self::Left),
            FrameAlign::Right => Ok(Self::Right),
            FrameAlign::Middle => Ok(Self::Middle),
            FrameAlign::Client => Err(Self::Error::new("Client")),
        }
    }
}

/// Alignment and position of a [`Frame`], which may either be an external (floating) frame or
/// an internal (docked) frame.
///
/// See [MXP specification: `<FRAME>`] and the additions documented in [`MUD Standards: Frames`].
///
/// [MXP specification: `<FRAME>`]: https://www.zuggsoft.com/zmud/mxp.htm#Frames
/// [`MUD Standards: Frames`]: https://mudstandards.org/mud/mxp/#frames
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameLayout<S> {
    /// Specifies that the frame is a floating frame.
    External {
        /// The coordinate of the left side of the frame. If a negative number is used, it means
        /// the value is relative to the right side of the screen instead of the left.
        left: Dimension<i32>,
        /// The coordinate of the top of the frame. If a negative value is used, it means the frame
        /// is relative to the bottom of the screen instead of the top.
        top: Dimension<i32>,
        /// The width of the frame. Percentage is calculated relative to screen width.
        width: Option<Dimension<u32>>,
        /// The height of the frame. Percentage is calculated relative to screen height.
        height: Option<Dimension<u32>>,
        /// Forces the frame to "stay on top" of the main MUD window.
        floating: bool,
    },
    /// Specifies that the frame is internal to another window.
    Internal {
        /// Specifies how the frame is docked with the MUD window.
        align: FrameAlign,
        /// The width of the frame. Percentage is calculated relative to main window width.
        width: Option<Dimension<u32>>,
        /// The height of the frame. Percentage is calculated relative to main window height.
        height: Option<Dimension<u32>>,
        /// Name of the window to dock into. If omitted, defaults to the main MUD window.
        dock: Option<S>,
    },
}

impl<S> Default for FrameLayout<S> {
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

impl<S> FrameLayout<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, f: F) -> FrameLayout<T>
    where
        F: FnOnce(S) -> T,
    {
        match self {
            Self::External {
                left,
                top,
                width,
                height,
                floating,
            } => FrameLayout::External {
                left,
                top,
                width,
                height,
                floating,
            },
            Self::Internal {
                align,
                width,
                height,
                dock,
            } => FrameLayout::Internal {
                align,
                width,
                height,
                dock: dock.map(f),
            },
        }
    }
}

impl<S: AsRef<str>> FrameLayout<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn borrow_text(&self) -> FrameLayout<&str> {
        match *self {
            Self::External {
                left,
                top,
                width,
                height,
                floating,
            } => FrameLayout::External {
                left,
                top,
                width,
                height,
                floating,
            },
            Self::Internal {
                align,
                width,
                height,
                ref dock,
            } => FrameLayout::Internal {
                align,
                width,
                height,
                dock: dock.as_ref().map(AsRef::as_ref),
            },
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
/// use mxp::{FrameAlign, Dimension, FrameAction, FrameLayout};
///
/// assert_eq!(
///     "<FRAME NAME=Map LEFT=-20c TOP=0 WIDTH=20c HEIGHT=20c>".parse::<mxp::Frame>(),
///     Ok(mxp::Frame {
///         name: "Map".into(),
///         action: FrameAction::Open,
///         title: "Map".into(),
///         scrolling: false,
///         persistent: false,
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
///         persistent: false,
///         layout: FrameLayout::Internal {
///             align: FrameAlign::Top,
///             width: None,
///             height: None,
///             dock: None,
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
    pub layout: FrameLayout<S>,
    /// Determines whether the frame is allowed to scroll.
    pub scrolling: bool,
    /// Existing windows/frames are not changed in size or position. So the specified
    /// top/left/width/height attributes only take effect when creating a new frame.
    pub persistent: bool,
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
            layout: self.layout.map_text(f),
            scrolling: self.scrolling,
            persistent: self.persistent,
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
            layout: self.layout.borrow_text(),
            scrolling: self.scrolling,
            persistent: self.persistent,
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

impl<'a> Frame<Cow<'a, str>> {
    pub(crate) fn scan<A>(scanner: A) -> crate::Result<Self>
    where
        A: ArgumentScanner<'a>,
    {
        let mut scanner = scanner.with_keywords();
        let name = scanner.decode_next_or("name")?.expect_some("name")?;
        let action = scanner
            .decode_next_or("action")?
            .expect_variant()?
            .unwrap_or_default();
        let title = scanner
            .decode_next_or("title")?
            .unwrap_or_else(|| name.clone());
        let align = scanner
            .decode_next_or("align")?
            .expect_variant()?
            .unwrap_or_default();
        let left = scanner
            .decode_next_or("left")?
            .expect_number()?
            .unwrap_or_default();
        let top = scanner
            .decode_next_or("top")?
            .expect_number()?
            .unwrap_or_default();
        let width = scanner.decode_next_or("width")?.expect_number()?;
        let height = scanner.decode_next_or("height")?.expect_number()?;
        let scrolling =
            scanner.decode_next_or("scrolling")?.expect_variant()? == Some(YesOrNo::Yes);
        let dock = scanner.decode_next_or("dock")?;
        let keywords = scanner.into_keywords()?;
        let layout = if keywords.contains(FrameKeyword::Internal) || dock.is_some() {
            FrameLayout::Internal {
                dock,
                align,
                width,
                height,
            }
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
            persistent: keywords.contains(FrameKeyword::Persistent),
        })
    }
}

impl_from_str!(Frame);

impl<S: AsRef<str>> fmt::Display for Frame<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Frame {
            name,
            action,
            title,
            layout,
            scrolling,
            persistent,
        } = self.borrow_text().map_text(crate::display::Escape);
        write!(f, "<FRAME NAME={name}")?;
        match action {
            FrameAction::Close => f.write_str(" CLOSE")?,
            FrameAction::Open => (),
            FrameAction::Redirect => f.write_str(" REDIRECT")?,
        }
        if title != name {
            write!(f, " TITLE={title}")?;
        }
        match layout {
            FrameLayout::External {
                left,
                top,
                width,
                height,
                floating,
            } => {
                if left.amount != 0 {
                    write!(f, " LEFT={left}")?;
                }
                if top.amount != 0 {
                    write!(f, " TOP={top}")?;
                }
                if let Some(width) = width {
                    write!(f, " WIDTH={width}")?;
                }
                if let Some(height) = height {
                    write!(f, " HEIGHT={height}")?;
                }
                if floating {
                    f.write_str(" FLOATING")?;
                }
            }
            FrameLayout::Internal {
                align,
                width,
                height,
                dock,
            } => {
                if align != FrameAlign::default() {
                    write!(f, " ALIGN={align}")?;
                }
                if let Some(width) = width {
                    write!(f, " WIDTH={width}")?;
                }
                if let Some(height) = height {
                    write!(f, " HEIGHT={height}")?;
                }
                match dock {
                    Some(dock) => write!(f, " DOCK={dock}")?,
                    None => write!(f, " INTERNAL")?,
                }
            }
        }
        if scrolling {
            f.write_str(" SCROLLING=yes")?;
        }
        if persistent {
            f.write_str(" PERSISTENT")?;
        }
        f.write_str(">")
    }
}
