use flagset::{FlagSet, flags};

use super::Negotiate;
use crate::transformer::TransformerConfig;

/// Negotiate About Character Set
pub const CODE: u8 = 42;

flags! {
    #[derive(PartialOrd, Ord, Hash)]
    enum Charset: u8 {
        Ascii,
        Utf8
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Charsets {
    inner: FlagSet<Charset>,
}

impl<T: AsRef<[u8]>> From<T> for Charsets {
    fn from(value: T) -> Self {
        // Reduce monomorphization
        fn inner(data: &[u8]) -> Charsets {
            if data.len() < 3 || data[0] != 1 {
                return Charsets::default();
            }
            let mut flags = FlagSet::default();
            let delim = data[1];
            for fragment in data[2..].split(|&c| c == delim) {
                if fragment == b"UTF-8" {
                    flags |= Charset::Utf8;
                } else if fragment == b"US-ASCII" {
                    flags |= Charset::Ascii;
                }
            }
            Charsets { inner: flags }
        }
        inner(value.as_ref())
    }
}

impl Charsets {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Negotiate for Charsets {
    const CODE: u8 = CODE;

    fn negotiate(self, buf: &mut Vec<u8>, config: &TransformerConfig) {
        if !config.disable_utf8 && self.inner.contains(Charset::Utf8) {
            buf.extend_from_slice(b"\x02UTF-8");
        } else if self.inner.contains(Charset::Ascii) {
            buf.extend_from_slice(b"\x02US-ASCII");
        } else {
            buf.push(3);
        }
    }
}
