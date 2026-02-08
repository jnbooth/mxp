use std::fmt;

use flagset::{FlagSet, flags};

use super::{Negotiate, mtts};
use crate::transformer::TransformerConfig;

/// MUD New-Environ Standard
/// https://tintin.mudhalla.net/protocols/mnes/
pub const CODE: u8 = 39;

#[allow(unused)]
pub const IS: u8 = 0;
pub const SEND: u8 = 1;
#[allow(unused)]
pub const INFO: u8 = 2;

pub const VAR: u8 = 0;
#[allow(unused)]
pub const VAL: u8 = 1;

flags! {
    enum KnownVariable: u8 {
        Charset,
        ClientName,
        ClientVersion,
        Mtts,
        TerminalType,
    }
}

impl KnownVariable {
    pub const fn parse(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"CHARSET" => Some(Self::Charset),
            b"CLIENT_NAME" => Some(Self::ClientName),
            b"CLIENT_VERSION" => Some(Self::ClientVersion),
            b"MTTS" => Some(Self::Mtts),
            b"TERMINAL_TYPE" => Some(Self::TerminalType),
            _ => None,
        }
    }

    fn negotiate<W: fmt::Write>(self, mut f: W, config: &TransformerConfig) -> fmt::Result {
        match self {
            Self::Charset => {
                if config.disable_utf8 {
                    f.write_str("\0CHARSET\x01ASCII")
                } else {
                    f.write_str("\0CHARSET\x01UTF-8")
                }
            }
            Self::ClientName => write!(f, "\0CLIENT_NAME\x01{}", config.terminal_identification),
            Self::ClientVersion => write!(f, "\0CLIENT_VERSION\x01{}", config.version),
            Self::Mtts => write!(f, "\0MTTS\x01{}", mtts::bitmask(config)),
            Self::TerminalType => write!(f, "\0TERMINAL_TYPE\x01{}", mtts::ttype(config)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Variables {
    inner: FlagSet<KnownVariable>,
    prefix: &'static str,
}

impl Default for Variables {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<T> for Variables
where
    T: AsRef<[u8]>,
{
    fn from(value: T) -> Self {
        // Reduce monomorphization
        fn inner(value: &[u8]) -> Variables {
            let mut inner = FlagSet::default();
            inner.extend(value.split(|&c| c == VAR).filter_map(KnownVariable::parse));
            Variables {
                inner,
                prefix: "\0", // IS
            }
        }

        inner(value.as_ref())
    }
}

impl Variables {
    pub const fn new() -> Self {
        Self {
            inner: FlagSet::empty(),
            prefix: "\0",
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

        if self.inner.contains(KnownVariable::Charset) && a.disable_utf8 != b.disable_utf8 {
            changes |= KnownVariable::Charset;
        }
        if self.inner.contains(KnownVariable::ClientName)
            && a.terminal_identification != b.terminal_identification
        {
            changes |= KnownVariable::ClientName;
        }
        if self.inner.contains(KnownVariable::ClientVersion) && a.version != b.version {
            changes |= KnownVariable::ClientVersion;
        }
        if self.inner.contains(KnownVariable::Mtts) && mtts::bitmask(a) != mtts::bitmask(b) {
            changes |= KnownVariable::Mtts;
        }
        if self.inner.contains(KnownVariable::TerminalType) && mtts::ttype(a) != mtts::ttype(b) {
            changes |= KnownVariable::TerminalType;
        }

        Self {
            inner: changes,
            prefix: "\x02", // INFO
        }
    }
}

impl Negotiate for Variables {
    const CODE: u8 = CODE;

    fn negotiate<W: fmt::Write>(self, mut f: W, config: &TransformerConfig) -> fmt::Result {
        f.write_str(self.prefix)?;
        for variable in self.inner {
            variable.negotiate(&mut f, config)?;
        }
        Ok(())
    }
}
