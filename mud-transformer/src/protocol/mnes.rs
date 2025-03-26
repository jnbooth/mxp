use flagset::{flags, FlagSet};
use std::fmt::{self, Display, Formatter};

use super::{mtts, Negotiate};
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

    pub fn fmt(self, formatter: &mut Formatter, config: &TransformerConfig) -> fmt::Result {
        match self {
            Self::Mtts => write!(formatter, "\x00MTTS\x01{}", mtts::bitmask(config)),
            Self::Charset => {
                if config.disable_utf8 {
                    formatter.write_str("\x00CHARSET\x01ASCII")
                } else {
                    formatter.write_str("\x00CHARSET\x01UTF-8")
                }
            }
            Self::ClientName => write!(
                formatter,
                "\x00CLIENT_NAME\x01{}",
                config.terminal_identification
            ),
            Self::ClientVersion => write!(formatter, "\x00CLIENT_VERSION\x01{}", config.version),
            Self::TerminalType => formatter.write_str("\x00TERMINAL_TYPE\x01ANSI-TRUECOLOR"),
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
    pub fn new() -> Self {
        Self {
            inner: FlagSet::default(),
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Subnegotiation<'a> {
    config: &'a TransformerConfig,
    prefix: &'a str,
    variables: FlagSet<Variable>,
}

impl<'a> Display for Subnegotiation<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.prefix)?;
        for variable in self.variables {
            variable.fmt(f, self.config)?;
        }
        Ok(())
    }
}

impl Negotiate for Variables {
    const CODE: u8 = CODE;

    type Output<'a> = Subnegotiation<'a>;

    fn negotiate(self, config: &TransformerConfig) -> Subnegotiation {
        Subnegotiation {
            config,
            prefix: self.prefix,
            variables: self.inner,
        }
    }
}
