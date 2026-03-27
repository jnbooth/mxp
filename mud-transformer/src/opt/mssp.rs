use std::iter::FusedIterator;
use std::{fmt, slice};

use crate::count_bytes;

/// MUD Server Status Protocol
///
/// https://tintin.mudhalla.net/protocols/mssp/
pub const OPT: u8 = 70;

pub const VAR: u8 = 1;
pub const VAL: u8 = 2;

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct Iter<'a> {
    data: slice::Iter<'a, u8>,
}

pub fn decode(data: &[u8]) -> Iter<'_> {
    match data {
        [VAR, rest @ ..] => Iter { data: rest.iter() },
        _ => Iter { data: [].iter() },
    }
}

impl<'a> Iter<'a> {
    fn slice_before(&mut self, c: u8) -> Result<&'a [u8], &'a [u8]> {
        let slice = self.data.as_slice();
        match self.data.position(|&ch| ch == c) {
            Some(pos) => Ok(&slice[..pos]),
            None => Err(slice),
        }
    }

    fn slice_after(&mut self, c: u8) -> Result<&'a [u8], &'a [u8]> {
        let slice = self.data.as_slice();
        match self.data.rposition(|&ch| ch == c) {
            Some(pos) => Ok(&slice[pos + 1..]),
            None => Err(slice),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a [u8], &'a [u8]);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let var = self.slice_before(VAL).ok()?;
        let val = match self.slice_before(VAR) {
            Ok(val) | Err(val) => val,
        };
        Some((var, val))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.len();
        (exact, Some(exact))
    }
}

impl DoubleEndedIterator for Iter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let val = self.slice_after(VAL).ok()?;
        let var = match self.slice_after(VAR) {
            Ok(var) | Err(var) => var,
        };
        Some((var, val))
    }
}

impl ExactSizeIterator for Iter<'_> {
    #[inline]
    fn len(&self) -> usize {
        count_bytes(self.data.as_slice(), VAL)
    }
}

impl FusedIterator for Iter<'_> {}

impl fmt::Debug for Iter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.clone()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mssp_iter_forward() {
        let data = b"\x01first\x02second\x01third\x02fourth";
        let values: Vec<_> = decode(data)
            .map(|(x, y)| {
                (
                    String::from_utf8_lossy(x).into_owned(),
                    String::from_utf8_lossy(y).into_owned(),
                )
            })
            .collect();
        assert_eq!(
            values,
            &[
                ("first".to_owned(), "second".to_owned()),
                ("third".to_owned(), "fourth".to_owned()),
            ]
        );
    }

    #[test]
    fn mssp_iter_back() {
        let data = b"\x01first\x02second\x01third\x02fourth";
        let values: Vec<_> = decode(data)
            .rev()
            .map(|(x, y)| {
                (
                    String::from_utf8_lossy(x).into_owned(),
                    String::from_utf8_lossy(y).into_owned(),
                )
            })
            .collect();
        assert_eq!(
            values,
            &[
                ("third".to_owned(), "fourth".to_owned()),
                ("first".to_owned(), "second".to_owned()),
            ]
        );
    }
}
