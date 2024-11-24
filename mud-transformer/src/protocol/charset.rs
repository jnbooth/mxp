use enumeration::{Enum, EnumSet};
use std::fmt::{self, Display, Formatter};

use crate::TransformerConfig;

/// Negotiate About Character Set
pub const CODE: u8 = 42;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
enum Charset {
    Ascii,
    Utf8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Charsets {
    inner: EnumSet<Charset>,
}

impl Default for Charsets {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: AsRef<[u8]>> From<T> for Charsets {
    fn from(value: T) -> Self {
        let mut inner = EnumSet::new();
        let data = value.as_ref();
        if data.len() < 3 || data[0] != 1 {
            return Self { inner };
        }
        let delim = data[1];
        for fragment in data[2..].split(|&c| c == delim) {
            if fragment == b"UTF-8" {
                inner.insert(Charset::Utf8);
            } else if fragment == b"US-ASCII" {
                inner.insert(Charset::Ascii);
            }
        }
        Self { inner }
    }
}

impl Charsets {
    pub const fn new() -> Self {
        Self {
            inner: EnumSet::new(),
        }
    }

    pub fn subnegotiation(self, config: &TransformerConfig) -> Subnegotiation {
        Subnegotiation {
            charsets: self.inner,
            utf8: !config.disable_utf8,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Subnegotiation {
    charsets: EnumSet<Charset>,
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
