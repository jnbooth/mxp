use std::fmt;
use std::num::NonZero;

use crate::arguments::{ArgumentScanner, ExpectArg, FromArgs};
use crate::keyword::DestKeyword;

/// Positions text at a certain position in a frame.
///
/// Note that when text in a frame or window scrolls, the text is no longer at the same X or Y position. So, for status windows, ensure that you set the frame to be unscrollable.
///
/// See [MXP specification: `<DEST>`](https://www.zuggsoft.com/zmud/mxp.htm#Cursor%20Control).
///
/// # Examples
///
/// ```
/// assert_eq!(
///     "<DEST status X=10 Y=2>".parse::<mxp::Dest>(),
///     Ok(mxp::Dest {
///         name: Some("status".into()),
///         column: 10.try_into().ok(),
///         line: 2.try_into().ok(),
///         eof: false,
///         eol: false,
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Dest<S = String> {
    /// Name of [frame](crate::Frame) to use as the destination for text.
    /// If unspecified, the text is sent to the main MUD window.
    pub name: Option<S>,
    /// Optional column in the frame to use as the position.
    pub column: Option<NonZero<u32>>,
    /// Optional line in the frame to use as the position.
    pub line: Option<NonZero<u32>>,
    /// Causes the rest of the frame to be erased after displaying the text.
    pub eof: bool,
    /// Causes the rest of the line to be erased after displaying the text.
    pub eol: bool,
}

impl<S> Dest<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, f: F) -> Dest<T>
    where
        F: FnOnce(S) -> T,
    {
        Dest {
            name: self.name.map(f),
            column: self.column,
            line: self.line,
            eof: self.eof,
            eol: self.eol,
        }
    }
}

impl_into_owned!(Dest);

impl<S: AsRef<str>> Dest<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Dest<&str> {
        Dest {
            name: self.name.as_ref().map(AsRef::as_ref),
            column: self.column,
            line: self.line,
            eof: self.eof,
            eol: self.eol,
        }
    }
}

impl_partial_eq!(Dest);

impl<S> From<Option<S>> for Dest<S> {
    fn from(name: Option<S>) -> Self {
        Self {
            name,
            column: None,
            line: None,
            eof: false,
            eol: false,
        }
    }
}

impl<S: AsRef<str>> From<S> for Dest<S> {
    fn from(name: S) -> Self {
        if name.as_ref().is_empty() {
            Self::from(None)
        } else {
            Self::from(Some(name))
        }
    }
}

impl<'a, S: AsRef<str>> FromArgs<'a, S> for Dest<S> {
    fn from_args<A: ArgumentScanner<'a, Decoded = S>>(scanner: A) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        let name = scanner.get_next()?;
        let column = scanner
            .get_next_or("x")?
            .expect_number()?
            .and_then(NonZero::new);
        let line = scanner
            .get_next_or("y")?
            .expect_number()?
            .and_then(NonZero::new);
        let keywords = scanner.into_keywords()?;
        Ok(Self {
            name,
            column,
            line,
            eof: keywords.contains(DestKeyword::Eof),
            eol: keywords.contains(DestKeyword::Eol),
        })
    }
}

impl_from_str!(Dest);

impl<S: AsRef<str>> fmt::Display for Dest<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Dest {
            name,
            column,
            line,
            eof,
            eol,
        } = self.borrow_text();
        crate::display::ElementFormatter {
            name: "DEST",
            arguments: &[&name, &column, &line],
            keywords: &[("EOF", eof), ("EOL", eol)],
        }
        .fmt(f)
    }
}
