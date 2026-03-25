use std::iter::FusedIterator;
use std::slice;

use crate::{Error, ErrorKind};

/// Iterator over the word units of an MXP string, converted into named or unnamed arguments.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub(crate) struct ArgumentParser<'a> {
    inner: slice::Iter<'a, u8>,
}

impl<'a> ArgumentParser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: source.as_bytes().iter(),
        }
    }

    fn next_arg(&mut self, positional: bool) -> Option<(&'a [u8], bool)> {
        let mut slice = self.inner.as_slice();
        if positional {
            let start = self.inner.position(|&c| c != b' ')?;
            slice = &slice[start..];
        } else {
            self.inner.next();
        }
        let c = *slice.first()?;
        if c == b' ' {
            return None;
        }
        if c == b'"' || c == b'\'' {
            let slice = match self.inner.position(|&ch| ch == c) {
                Some(pos) => &slice[1..=pos],
                None => &slice[1..],
            };
            return Some((slice, false));
        }
        let break_condition = if positional {
            |&ch| ch == b' ' || ch == b'='
        } else {
            |&ch| ch == b' '
        };
        let Some(breakpoint) = self.inner.position(break_condition) else {
            return Some((slice, false));
        };
        Some((&slice[..=breakpoint], slice[breakpoint + 1] == b'='))
    }
}

impl<'a> Iterator for ArgumentParser<'a> {
    type Item = crate::Result<(&'a str, Option<&'a str>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let (slice1, is_named) = self.next_arg(true)?;
        // SAFETY: Valid UTF-8.
        let arg1 = unsafe { str::from_utf8_unchecked(slice1) };
        if matches!(slice1, [b'=', ..]) {
            return Some(Err(Error::new(arg1, ErrorKind::MissingArgumentName)));
        }
        if !is_named {
            return Some(Ok((arg1, None)));
        }
        let Some((slice2, _)) = self.next_arg(false) else {
            let target = format!("{arg1}=");
            return Some(Err(Error::new(target, ErrorKind::EmptyArgument)));
        };
        // SAFETY: Valid UTF-8.
        let arg2 = unsafe { str::from_utf8_unchecked(slice2) };
        Some(Ok((arg1, Some(arg2))))
    }

    // A generous size hint reflecting the total number of spaces in the string.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl FusedIterator for ArgumentParser<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args() {
        let args =
            ArgumentParser::new("  EL      RName  '<FONT COLOR=Red><B>' FLAG=\"RoomName\"  ")
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
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
