use std::borrow::Cow;

use crate::arguments::{ArgumentScanner, ExpectArg as _};
use crate::parse::Decoder;

/// Defines a graphics format and provides a client plugin module that converts the MUD-specific
/// format to a standard GIF or BMP format.
///
/// See [MXP specification: `<FILTER>`](https://www.zuggsoft.com/zmud/mxp.htm#File%20Filters).
///
/// # Examples
///
/// ```
/// assert_eq!(
///     "<FILTER SRC='bff' DEST='gif' NAME='MyPlugin'>".parse::<mxp::Filter>(),
///     Ok(mxp::Filter {
///         src: "bff".into(),
///         dest: Some("gif".into()),
///         name: "MyPlugin".into(),
///         proc: 0,
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Filter<S = String> {
    /// File extension of the MUD-specific format.
    pub src: S,
    /// Output file extension. Default is BMP.
    pub dest: Option<S>,
    /// Name of the plugin to be called.
    pub name: S,
    /// Numeric parameter that the plugin can use to support multiple conversions as needed.
    /// Default is 0.
    pub proc: u32,
}

impl<S> Filter<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, mut f: F) -> Filter<T>
    where
        F: FnMut(S) -> T,
    {
        Filter {
            src: f(self.src),
            dest: self.dest.map(&mut f),
            name: f(self.name),
            proc: self.proc,
        }
    }
}

impl_into_owned!(Filter);

impl<S: AsRef<str>> Filter<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Filter<&str> {
        Filter {
            src: self.src.as_ref(),
            dest: self.dest.as_ref().map(AsRef::as_ref),
            name: self.name.as_ref(),
            proc: self.proc,
        }
    }
}

impl_partial_eq!(Filter);

impl<S: AsRef<str>> Filter<S> {
    pub(crate) fn scan<A>(mut scanner: A) -> crate::Result<Self>
    where
        A: ArgumentScanner<Output = S>,
    {
        let src = scanner.next_or("src")?.expect_some("src")?;
        let dest = scanner.next_or("dest")?;
        let name = scanner.next_or("name")?.expect_some("name")?;
        let proc = scanner.next_or("proc")?.expect_number()?.unwrap_or(0);
        scanner.expect_end()?;
        Ok(Self {
            src,
            dest,
            name,
            proc,
        })
    }
}

impl_from_str!(Filter);
