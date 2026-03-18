use std::cell::Cell;
use std::fmt;
use std::num::NonZero;

use flagset::FlagSet;

use crate::{Dimension, RgbColor};

fn is_plain_word(s: &str) -> bool {
    const fn is_wordlike(c: u8) -> bool {
        matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'-')
    }
    s.bytes().all(is_wordlike)
}

pub(crate) struct DelimAfterFirst {
    delim: &'static str,
    after_first: Cell<bool>,
}

impl DelimAfterFirst {
    pub const fn new(delim: &'static str) -> Self {
        Self {
            delim,
            after_first: Cell::new(false),
        }
    }
}

impl fmt::Display for DelimAfterFirst {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.after_first.replace(true) {
            f.write_str(self.delim)
        } else {
            Ok(())
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Escape<'a>(pub &'a str);

impl fmt::Display for Escape<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if is_plain_word(self.0) {
            return self.0.fmt(f);
        }
        let escaped = html_escape::encode_double_quoted_attribute(self.0);
        write!(f, "\"{escaped}\"")
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MaybeQuote<'a>(pub &'a str);

impl fmt::Display for MaybeQuote<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let body = self
            .0
            .strip_prefix('&')
            .and_then(|s| s.strip_suffix(';'))
            .unwrap_or(self.0);
        if is_plain_word(body) {
            self.0.fmt(f)
        } else {
            write!(f, "\"{}\"", self.0)
        }
    }
}

pub(crate) trait DisplayArg {
    fn is_default(&self) -> bool {
        false
    }
    fn display(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

struct DisplayAsArg<'a>(&'a dyn DisplayArg);
impl fmt::Display for DisplayAsArg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.display(f)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct ElementFormatter<'a> {
    pub name: &'a str,
    pub arguments: &'a [&'a dyn DisplayArg],
    pub keywords: &'a [(&'a str, bool)],
}

impl fmt::Display for ElementFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            name,
            arguments,
            keywords,
        } = *self;
        let args = match arguments.iter().rposition(|arg| !arg.is_default()) {
            Some(i) => &arguments[..=i],
            None => &[],
        };

        if args.is_empty() && keywords.iter().all(|k| !k.1) {
            return write!(f, "<{name}>");
        }

        write!(f, "<{name}")?;
        for &arg in args {
            write!(f, " {}", DisplayAsArg(arg))?;
        }
        for &(keyword, set) in keywords {
            if set {
                write!(f, " {keyword}")?;
            }
        }
        f.write_str(">")?;
        Ok(())
    }
}

macro_rules! impl_display {
    ($t:ty) => {
        impl DisplayArg for $t {
            fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(self, f)
            }
        }
    };
}

impl_display!(RgbColor);
impl_display!(u8);
impl_display!(u16);
impl_display!(u32);
impl_display!(NonZero<u8>);
impl_display!(NonZero<u32>);
impl_display!(crate::screen::Align);
impl_display!(crate::elements::AudioRepetition);

impl DisplayArg for (Option<RgbColor>, FlagSet<crate::FontStyle>) {
    fn is_default(&self) -> bool {
        self.0.is_none() && self.1.is_empty()
    }

    fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let delim = DelimAfterFirst::new(",");
        if let Some(color) = self.0 {
            write!(f, "{color}{delim}")?;
        }
        for style in self.1 {
            write!(f, "{delim}{style}")?;
        }
        Ok(())
    }
}

impl DisplayArg for &str {
    fn is_default(&self) -> bool {
        self.is_empty()
    }

    fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&Escape(self), f)
    }
}

impl<T: DisplayArg> DisplayArg for Option<T> {
    fn is_default(&self) -> bool {
        self.is_none()
    }

    fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Some(arg) => arg.display(f),
            None => f.write_str("\"\""),
        }
    }
}

impl<T: fmt::Display> DisplayArg for Dimension<T> {
    fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl<T: DisplayArg + Eq> DisplayArg for (T, T) {
    fn is_default(&self) -> bool {
        self.0 == self.1
    }

    fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.display(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delim_after_first() {
        let delim = DelimAfterFirst::new(", ");
        let output = format!("{delim}A{delim}B{delim}C");
        assert_eq!(output, "A, B, C");
    }

    #[test]
    fn escape() {
        let output = Escape("this & \"that\"").to_string();
        assert_eq!(output, "\"this &amp; &quot;that&quot;\"");
    }
}
