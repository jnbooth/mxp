use std::iter::FusedIterator;
use std::slice;

use super::arguments::Arguments;
use super::error::{Error, ErrorKind};
use super::validation::validate;

/// Iterator over the word units of an MXP string.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Words<'a> {
    iter: slice::Iter<'a, u8>,
    done: bool,
    source: &'a str,
}

impl<'a> Words<'a> {
    pub fn new(source: &'a str) -> Self {
        let source = source.trim_ascii();
        let mut iter = source.as_bytes().iter();
        Self {
            done: iter.next().is_none(),
            iter,
            source,
        }
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    fn get_offset(&self) -> usize {
        self.source.len() - self.iter.len() - usize::from(!self.done)
    }

    pub fn as_str(&self) -> &'a str {
        &self.source[self.get_offset()..]
    }

    pub fn validate_next_or(&mut self, e: ErrorKind) -> crate::Result<&'a str> {
        match self.next() {
            None => Err(Error::new(self.source, e)),
            Some(next) => {
                validate(next, e)?;
                Ok(next)
            }
        }
    }

    pub fn parse_args(self) -> crate::Result<Arguments<'a>> {
        self.try_into()
    }

    pub fn parse_args_to_owned(self) -> crate::Result<Arguments<'static>> {
        let mut arguments = Arguments::new();
        arguments.extend::<String>(self)?;
        Ok(arguments)
    }
}

const fn is_decimal(c: u8) -> bool {
    matches!(c, b'0'..=b'9' | b','..=b'.' | b'_')
}
const fn is_ident(c: u8) -> bool {
    matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b','..=b'.' | b'_')
}
const fn is_utf8_continuation(c: u8) -> bool {
    (c & 0xC0) == 0x80
}

impl<'a> Iterator for Words<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut offset = self.get_offset();
        let first = *self.source.as_bytes().get(offset)?;
        let mut finished_quote = false;
        self.done = !match first {
            b'"' | b'\'' => {
                offset += 1;
                finished_quote = self.iter.any(|&c| c == first);
                self.iter.next().is_some()
            }
            b'#' => self.iter.any(|&c| !c.is_ascii_hexdigit()),
            b'&' => self.iter.any(|&c| c == b';') && self.iter.next().is_some(),
            b'0'..=b'9' | b'+' | b'-' => self.iter.any(|&c| !is_decimal(c)),
            b'A'..=b'Z' | b'a'..=b'z' => self.iter.any(|&c| !is_ident(c)),
            128.. => self.iter.any(|&c| !is_utf8_continuation(c)),
            _ => self.iter.next().is_some(),
        };
        let mut end = self.get_offset();
        if let Some(next) = self.source.as_bytes().get(end)
            && next.is_ascii_whitespace()
        {
            self.iter.any(|&c| !c.is_ascii_whitespace());
        }
        if finished_quote {
            end -= 1;
        }
        // SAFETY: `offset` and `end` are both valid character boundaries.
        Some(unsafe { self.source.get_unchecked(offset..end) })
    }
}

impl FusedIterator for Words<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn show_words<'a, I>(words: I) -> String
    where
        I: IntoIterator<Item = &'a str>,
    {
        use std::fmt::Write;
        let mut buf = String::new();
        for word in words {
            write!(buf, "<{word}> ").unwrap();
        }
        buf
    }

    #[test]
    fn words() {
        let unwords = "   'teseeeeeet'     #123a56f89&aaabcdeef;foo,woo!-2.5,3_1a =- t🥀e''a{";
        let words = vec![
            "teseeeeeet",
            "#123a56f89",
            "&aaabcdeef;",
            "foo,woo",
            "!",
            "-2.5,3_1",
            "a",
            "=",
            "-",
            "t",
            "🥀",
            "e",
            "",
            "a",
            "{",
        ];
        assert_eq!(show_words(Words::new(unwords)), show_words(words));
    }
}
