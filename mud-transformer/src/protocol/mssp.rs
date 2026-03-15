use std::fmt;

use bytes::{Buf, Bytes};

/// MUD Server Status Protocol
///
/// https://tintin.mudhalla.net/protocols/mssp/
pub const CODE: u8 = 70;

pub const VAR: u8 = 1;
pub const VAL: u8 = 2;

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub(crate) struct Iter {
    data: Bytes,
}

pub fn iter(data: &Bytes) -> Iter {
    let data = match data.iter().position(|&c| c == VAR) {
        Some(i) => data.slice(i + 1..),
        None => Bytes::new(),
    };
    Iter { data }
}

fn split_until(bytes: &mut Bytes, delim: u8) -> Option<Bytes> {
    let i = bytes.iter().position(|&c| c == delim)?;
    let split = bytes.split_to(i);
    bytes.advance(1);
    Some(split)
}

impl Iterator for Iter {
    type Item = (Bytes, Bytes);

    fn next(&mut self) -> Option<Self::Item> {
        let before = split_until(&mut self.data, VAL)?;
        match split_until(&mut self.data, VAR) {
            Some(after) => Some((before, after)),
            None => Some((before, self.data.split_off(0))),
        }
    }
}

impl fmt::Debug for Iter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn format(bytes: &Bytes) -> String {
            String::from_utf8_lossy(bytes).into_owned()
        }
        f.debug_map()
            .entries(self.clone().map(|(k, v)| (format(&k), format(&v))))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mssp_iter() {
        let data = Bytes::copy_from_slice(b"abc\x01first\x02second\x01third\x02fourth");
        let values: Vec<_> = iter(&data)
            .map(|(x, y)| {
                (
                    String::from_utf8_lossy(&x).into_owned(),
                    String::from_utf8_lossy(&y).into_owned(),
                )
            })
            .collect();
        assert_eq!(
            values,
            &[
                ("first".to_owned(), "second".to_owned()),
                ("third".to_owned(), "fourth".to_owned())
            ]
        );
    }
}
