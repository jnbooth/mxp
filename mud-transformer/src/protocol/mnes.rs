use std::fmt;

use flagset::{FlagSet, flags};

use super::{Negotiate, mtts};
use crate::transformer::TransformerConfig;

/// MUD New-Environ Standard
/// https://tintin.mudhalla.net/protocols/mnes/
pub const CODE: u8 = 39;

flags! {
    enum Variable: u8 {
        Charset,
        ClientName,
        ClientVersion,
        Mtts,
        TerminalType,
    }
}

impl Variable {
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
                    f.write_str("\x00CHARSET\x01ASCII")
                } else {
                    f.write_str("\x00CHARSET\x01UTF-8")
                }
            }
            Self::ClientName => write!(f, "\x00CLIENT_NAME\x01{}", config.terminal_identification),
            Self::ClientVersion => write!(f, "\x00CLIENT_VERSION\x01{}", config.version),
            Self::Mtts => write!(f, "\x00MTTS\x01{}", mtts::bitmask(config)),
            Self::TerminalType => write!(f, "\x00TERMINAL_TYPE\x01{}", mtts::ttype(config)),
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

impl<T> From<T> for Variables
where
    T: AsRef<[u8]>,
{
    fn from(value: T) -> Self {
        // Reduce monomorphization
        fn inner(value: &[u8]) -> Variables {
            let mut inner = FlagSet::default();
            inner.extend(value.split(|&c| c == 0).filter_map(Variable::parse));
            Variables {
                inner,
                prefix: "\0",
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
        if self.inner.contains(Variable::Mtts) && mtts::bitmask(a) != mtts::bitmask(b) {
            changes |= Variable::Mtts;
        }
        if self.inner.contains(Variable::TerminalType) && mtts::ttype(a) != mtts::ttype(b) {
            changes |= Variable::TerminalType;
        }

        Self {
            inner: changes,
            prefix: "\x02",
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
