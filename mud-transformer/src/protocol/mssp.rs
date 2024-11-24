use std::slice;

/// MUD Server Status Protocol
///
/// https://tintin.mudhalla.net/protocols/mssp/
pub const CODE: u8 = 70;

#[derive(Clone, Debug, Default)]
pub struct Iter<'a> {
    inner: slice::Iter<'a, u8>,
}

pub fn iter(subnegotiation: &[u8]) -> Iter {
    let mut inner = subnegotiation.iter();
    inner.position(|&c| c == 1);
    Iter { inner }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a [u8], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.inner.as_slice();
        let before = self.inner.position(|&c| c == 2)?;
        let after = self.inner.position(|&c| c == 1).unwrap_or(slice.len());
        Some((&slice[..before], &slice[before + 1..after]))
    }
}
