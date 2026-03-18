use std::borrow::Cow;
use std::fmt;

use crate::arguments::{ArgumentScanner, ExpectArg as _};
use crate::parse::Decoder;

/// Opens a web page in the user's web browser.
///
/// See [MXP specification: `<A>`](https://www.zuggsoft.com/zmud/mxp.htm#Links).
///
/// # Examples
///
/// ```
/// assert_eq!(
///     "<A 'http://github.com' EXPIRE=all>".parse::<mxp::Hyperlink>(),
///     Ok(mxp::Hyperlink {
///         href: "http://github.com".into(),
///         hint: "http://github.com".into(),
///         expire: Some("all".into()),
///     }),
/// );
///
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Hyperlink<S = String> {
    /// Link to open.
    pub href: S,
    /// Mouseover hint.
    pub hint: S,
    /// Optional scope for the link. If defined, an [`Expire`](crate::Expire) command can
    /// invalidate the link.
    pub expire: Option<S>,
}

impl<S> Hyperlink<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, mut f: F) -> Hyperlink<T>
    where
        F: FnMut(S) -> T,
    {
        Hyperlink {
            href: f(self.href),
            hint: f(self.hint),
            expire: self.expire.map(f),
        }
    }
}

impl_into_owned!(Hyperlink);

impl<S: AsRef<str>> Hyperlink<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Hyperlink<&str> {
        Hyperlink {
            href: self.href.as_ref(),
            hint: self.hint.as_ref(),
            expire: self.expire.as_ref().map(AsRef::as_ref),
        }
    }
}

impl_partial_eq!(Hyperlink);

impl<'a> Hyperlink<Cow<'a, str>> {
    pub(crate) fn scan<A>(mut scanner: A) -> crate::Result<Self>
    where
        A: ArgumentScanner<'a>,
    {
        let href = scanner.decode_next_or("href")?.expect_some("href")?;
        let hint = scanner
            .decode_next_or("hint")?
            .unwrap_or_else(|| href.clone());
        let expire = scanner.decode_next_or("expire")?;
        scanner.expect_end()?;
        Ok(Self { href, hint, expire })
    }
}

impl_from_str!(Hyperlink);

impl<S: AsRef<str>> fmt::Display for Hyperlink<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Hyperlink {
            href,
            mut hint,
            expire,
        } = self.borrow_text();
        if hint == href {
            hint = "";
        }
        crate::display::ElementFormatter {
            name: "A",
            arguments: &[&href, &hint, &expire],
            keywords: &[],
        }
        .fmt(f)
    }
}
