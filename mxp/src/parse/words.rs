use std::iter::FusedIterator;
use std::{fmt, slice};

use super::count_bytes;
use crate::arguments::Arguments;
use crate::{Error, ErrorKind};

/// Iterator over the word units of an MXP string.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub(crate) struct Words<'a> {
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

    pub fn next_or(&mut self, e: ErrorKind) -> crate::Result<&'a str> {
        match self.next() {
            Some(next) => Ok(next),
            None => Err(Error::new(self.source, e)),
        }
    }

    pub fn args(self) -> WordArguments<'a> {
        WordArguments::new(self)
    }

    pub fn parse_args(self) -> crate::Result<Arguments<'a>> {
        self.try_into()
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    pub fn as_str(&self) -> &'a str {
        &self.source[self.get_offset()..]
    }

    pub fn as_bytes(&self) -> &'a [u8] {
        &self.source.as_bytes()[self.get_offset()..]
    }

    pub fn next_byte(&self) -> Option<u8> {
        self.source.as_bytes().get(self.get_offset()).copied()
    }

    fn get_offset(&self) -> usize {
        self.source.len() - self.iter.len() - usize::from(!self.done)
    }

    fn iter_while<F: FnMut(&u8) -> bool>(&mut self, pred: F) {
        self.done = self.iter.all(pred);
    }

    fn iter_until(&mut self, c: u8) -> bool {
        let found = self.iter.any(|&ch| ch == c);
        self.iter_once();
        found
    }

    fn iter_once(&mut self) {
        self.done = self.iter.next().is_none();
    }

    fn get_byte(&self, i: usize) -> Option<u8> {
        self.source.as_bytes().get(i).copied()
    }
}

impl fmt::Debug for Words<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a> Iterator for Words<'a> {
    type Item = &'a str;

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn next(&mut self) -> Option<Self::Item> {
        const fn is_word(c: &u8) -> bool {
            matches!(*c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+'..=b'.' | b'_' | b'|')
        }
        const fn is_utf8_continuation(c: &u8) -> bool {
            (*c & 0xC0) == 0x80
        }

        let mut offset = self.get_offset();
        let first = self.get_byte(offset)?;
        let mut finished_quote = false;
        match first {
            b'"' | b'\'' => {
                offset += 1;
                finished_quote = self.iter_until(first);
            }
            b'#' => {
                self.iter_while(u8::is_ascii_hexdigit);
            }
            b'&' => {
                self.iter_until(b';');
            }
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'-' | b'|' => {
                self.iter_while(is_word);
            }
            128.. => {
                self.iter_while(is_utf8_continuation);
            }
            _ => self.iter_once(),
        }
        let mut end = self.get_offset();
        if let Some(next) = self.get_byte(end)
            && next.is_ascii_whitespace()
        {
            self.iter_while(u8::is_ascii_whitespace);
        }
        if finished_quote {
            end -= 1;
        }
        // SAFETY: `offset` and `end` are both valid character boundaries.
        Some(unsafe { self.source.get_unchecked(offset..end) })
    }

    // A generous size hint reflecting the total number of spaces in the string.
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(count_bytes(self.as_bytes(), b' ') + 1))
    }
}

impl FusedIterator for Words<'_> {}

/// Iterator over the word units of an MXP string, converted into named or unnamed arguments.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub(crate) struct WordArguments<'a> {
    inner: Words<'a>,
}

impl<'a> WordArguments<'a> {
    pub fn new(words: Words<'a>) -> Self {
        Self { inner: words }
    }
}

impl<'a> Iterator for WordArguments<'a> {
    type Item = crate::Result<(&'a str, Option<&'a str>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = self.inner.next()?;
        if name == "=" {
            return Some(Err(Error::new(
                format!("={}", self.inner.next().unwrap_or_default()),
                ErrorKind::MissingArgumentName,
            )));
        }
        if self.inner.next_byte() != Some(b'=') {
            return Some(Ok((name, None)));
        }
        self.inner.next();
        if let Some(val) = self.inner.next() {
            return Some(Ok((name, Some(val))));
        }
        Some(Err(Error::new(
            format!("{name}="),
            ErrorKind::EmptyArgument,
        )))
    }

    // A generous size hint reflecting the total number of spaces in the string.
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

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
            "-2.5,3_1a",
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

    #[test]
    fn args() {
        let words = Words::new("EL RName '<FONT COLOR=Red><B>' FLAG=\"RoomName\"");
        let args = words.args().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(
            args,
            &[
                ("EL", None),
                ("RName", None),
                ("<FONT COLOR=Red><B>", None),
                ("FLAG", Some("RoomName")),
            ]
        );
    }
}
