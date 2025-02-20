use flagset::{flags, FlagSet};
use std::fmt::{self, Display, Formatter};

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
pub struct Charsets {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Subnegotiation {
    charsets: FlagSet<Charset>,
    utf8: bool,
}

impl Display for Subnegotiation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let sequence = if self.utf8 && self.charsets.contains(Charset::Utf8) {
            "\x02UTF-8"
        } else if self.charsets.contains(Charset::Ascii) {
            "\x02US-ASCII"
        } else {
            "\x03"
        };
        f.write_str(sequence)
    }
}

impl Negotiate for Charsets {
    const CODE: u8 = CODE;

    type Output<'a> = Subnegotiation;

    fn negotiate(self, config: &TransformerConfig) -> Subnegotiation {
        Subnegotiation {
            charsets: self.inner,
            utf8: !config.disable_utf8,
        }
    }
}
