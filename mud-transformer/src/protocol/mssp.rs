use std::slice;

/// MUD Server Status Protocol
///
/// https://tintin.mudhalla.net/protocols/mssp/
pub const CODE: u8 = 70;

#[derive(Clone, Debug, Default)]
pub(crate) struct Iter<'a> {
    inner: slice::Iter<'a, u8>,
}

pub fn iter(subnegotiation: &[u8]) -> Iter<'_> {
    let mut inner = subnegotiation.iter();
    inner.position(|&c| c == 1);
    Iter { inner }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a [u8], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.inner.as_slice();
        let before = self.inner.position(|&c| c == 2)?;
        match self.inner.position(|&c| c == 1) {
            Some(len) => Some((&slice[..before], &slice[before + 1..before + 1 + len])),
            None => Some((&slice[..before], &slice[before + 1..])),
        }
    }
}
