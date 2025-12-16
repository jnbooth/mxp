use std::iter::FusedIterator;
use std::str::{self, CharIndices};

use super::error::{Error, ErrorKind};
use super::validation::validate;
use crate::argument::Arguments;

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Words<'a> {
    s: &'a str,
    iter: CharIndices<'a>,
    current: Option<(usize, char)>,
}

impl<'a> Words<'a> {
    pub fn new(s: &'a str) -> Self {
        let s = s.trim();
        let mut iter = s.char_indices();
        Self {
            current: iter.next(),
            iter,
            s,
        }
    }

    pub fn as_str(&self) -> &'a str {
        match self.current {
            None => "",
            Some((i, _)) => &self.s[i..],
        }
    }

    pub fn validate_next_or(&mut self, e: ErrorKind) -> crate::Result<&'a str> {
        match self.next() {
            None => Err(Error::new("", e)),
            Some(next) => {
                validate(next, e)?;
                Ok(next)
            }
        }
    }

    pub fn parse_args<S>(self) -> crate::Result<Arguments<S>>
    where
        S: AsRef<str> + From<&'a str>,
    {
        let mut args = Arguments::<S>::new();
        args.append(self)?;
        Ok(args)
    }
}

impl<'a> Iterator for Words<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        const fn is_non_decimal(c: char) -> bool {
            !c.is_ascii_digit() && c != '_' && c != '-' && c != '.' && c != ','
        }
        const fn is_non_alphabet(c: char) -> bool {
            !c.is_ascii_alphabetic() && !c.is_ascii_digit() && c != '_' && c != '-' && c != '.'
        }
        let (mut start, first) = self.current?;
        self.current = match first {
            // quoted string e.g. 'foo' or "foo"
            '\'' | '\"' => {
                start += 1; // skip opening quote
                self.iter.find(|&(_, c)| c == first);
                self.iter.next() // skip closing quote for next word
            }
            // hex color e.g. #xxxxxx
            '#' => self.iter.find(|&(_, c)| !c.is_ascii_hexdigit()),
            // argument e.g. &xxx;
            '&' => {
                self.iter.find(|&(_, c)| c == ';');
                self.iter.next() // inclusive range
            }
            // signed number e.g. -3,100.5
            '+' | '-' => self.iter.find(|&(_, c)| is_non_decimal(c)),
            // unsigned number e.g. 3,100.5
            _ if first.is_ascii_digit() => self.iter.find(|&(_, c)| is_non_decimal(c)),
            // word e.g. foo
            _ if first.is_ascii_alphabetic() => self.iter.find(|&(_, c)| is_non_alphabet(c)),
            // single character, e.g. = or ,
            _ => self.iter.next(),
        };
        let (mut end, nextchar) = match self.current {
            Some(x) => x,
            None if first == '"' || first == '\'' => {
                return Some(&self.s[start..self.s.len() - 1]);
            }
            None => {
                return Some(&self.s[start..]);
            }
        };
        if first == '"' || first == '\'' {
            end -= 1; // shrink back from quote
        }
        if nextchar.is_ascii_whitespace() {
            self.current = self.iter.find(|&(_, c)| !c.is_ascii_whitespace());
        }
        Some(&self.s[start..end])
    }
}

impl FusedIterator for Words<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words() {
        let unwords = "   'teseeeeeet'     #123a56f89&aaabcdeef;foo,-2.5,3_1a =- te''a{";
        let words = vec![
            "teseeeeeet",
            "#123a56f89",
            "&aaabcdeef;",
            "foo",
            ",",
            "-2.5,3_1",
            "a",
            "=",
            "-",
            "te",
            "",
            "a",
            "{",
        ];
        assert_eq!(Words::new(unwords).collect::<Vec<&str>>(), words);
    }
}
