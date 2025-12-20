use std::io::Write;

use flagset::{FlagSet, flags};

use super::{Negotiate, mtts};
use crate::transformer::TransformerConfig;

/// MUD New-Environ Standard
/// https://tintin.mudhalla.net/protocols/mnes/
pub const CODE: u8 = 39;

flags! {
    #[derive(PartialOrd, Ord, Hash)]
    enum Variable: u8 {
        Mtts,
        Charset,
        ClientName,
        ClientVersion,
        TerminalType,
    }
}

impl Variable {
    pub const fn parse(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"MTTS" => Some(Self::Mtts),
            b"CHARSET" => Some(Self::Charset),
            b"CLIENT_NAME" => Some(Self::ClientName),
            b"CLIENT_VERSION" => Some(Self::ClientVersion),
            b"TERMINAL_TYPE" => Some(Self::TerminalType),
            _ => None,
        }
    }

    fn negotiate(self, buf: &mut Vec<u8>, config: &TransformerConfig) {
        match self {
            Self::Mtts => {
                write!(buf, "\x00MTTS\x01{}", mtts::bitmask(config)).unwrap();
            }
            Self::Charset => {
                if config.disable_utf8 {
                    buf.extend_from_slice(b"\x00CHARSET\x01ASCII");
                } else {
                    buf.extend_from_slice(b"\x00CHARSET\x01UTF-8");
                }
            }
            Self::ClientName => {
                buf.extend_from_slice(b"\x00CLIENT_NAME\x01");
                buf.extend_from_slice(config.terminal_identification.as_bytes());
            }
            Self::ClientVersion => {
                buf.extend_from_slice(b"\x00CLIENT_VERSION\x01");
                buf.extend_from_slice(config.version.as_bytes());
            }
            Self::TerminalType => buf.extend_from_slice(b"\x00TERMINAL_TYPE\x01ANSI-TRUECOLOR"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Variables {
    inner: FlagSet<Variable>,
    prefix: &'static str,
}

impl Default for Variables {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: AsRef<[u8]>> From<T> for Variables {
    fn from(value: T) -> Self {
        let mut inner = FlagSet::default();
        inner.extend(
            value
                .as_ref()
                .split(|&c| c == 0)
                .filter_map(Variable::parse),
        );
        Self {
            inner,
            prefix: "\x00",
        }
    }
}

impl Variables {
    pub const fn new() -> Self {
        Self {
            inner: FlagSet::empty(),
            prefix: "\x00",
        }
    }

    pub fn is_empty(self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn changes(self, a: &TransformerConfig, b: &TransformerConfig) -> Self {
        let mut changes = FlagSet::default();

        if self.inner.contains(Variable::Mtts) && mtts::bitmask(a) != mtts::bitmask(b) {
            changes |= Variable::Mtts;
        }
        if self.inner.contains(Variable::Charset) && a.disable_utf8 != b.disable_utf8 {
            changes |= Variable::Charset;
        }
        if self.inner.contains(Variable::ClientName)
            && a.terminal_identification != b.terminal_identification
        {
            changes |= Variable::ClientName;
        }
        if self.inner.contains(Variable::ClientVersion) && a.version != b.version {
            changes |= Variable::ClientVersion;
        }

        Self {
            inner: changes,
            prefix: "\x02",
        }
    }
}

impl Negotiate for Variables {
    const CODE: u8 = CODE;

    fn negotiate(self, buf: &mut Vec<u8>, config: &TransformerConfig) {
        buf.extend_from_slice(self.prefix.as_bytes());
        for variable in self.inner {
            variable.negotiate(buf, config);
        }
    }
}
