use std::fmt;

use flagset::{FlagSet, flags};

use super::mtts;
use super::negotiate::Negotiate;
use crate::transformer::TransformerConfig;

/// MUD New-Environ Standard
/// https://tintin.mudhalla.net/protocols/mnes/
pub const OPT: u8 = 39;

pub const IS: u8 = 0;
pub const SEND: u8 = 1;
pub const INFO: u8 = 2;

pub const VAR: u8 = 0;
#[allow(unused)]
pub const VAL: u8 = 1;

flags! {
    enum KnownVariable: u8 {
        Charset,
        ClientName,
        ClientVersion,
        IpAddress,
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
            b"IPADDRESS" => Some(Self::IpAddress),
            b"MTTS" => Some(Self::Mtts),
            b"TERMINAL_TYPE" => Some(Self::TerminalType),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Charset => "CHARSET",
            Self::ClientName => "CLIENT_NAME",
            Self::ClientVersion => "CLIENT_VERSION",
            Self::IpAddress => "IPADDRESS",
            Self::Mtts => "MTTS",
            Self::TerminalType => "TERMINAL_TYPE",
        }
    }

    fn write<W: fmt::Write, T: fmt::Display>(self, mut f: W, val: T) -> fmt::Result {
        write!(f, "\0{}\x01{}", self.as_str(), val)
    }

    fn negotiate<W: fmt::Write>(self, f: W, config: &TransformerConfig) -> fmt::Result {
        match self {
            Self::Charset => {
                if config.disable_utf8 {
                    self.write(f, "ASCII")
                } else {
                    self.write(f, "UTF-8")
                }
            }
            Self::ClientName => self.write(f, &config.terminal_identification),
            Self::ClientVersion => self.write(f, &config.version),
            Self::IpAddress => {
                if let Some(client_ip) = &config.client_ip {
                    self.write(f, client_ip)
                } else {
                    Ok(())
                }
            }
            Self::Mtts => self.write(f, mtts::bitmask(config)),
            Self::TerminalType => self.write(f, mtts::ttype(config)),
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
        if self.inner.contains(KnownVariable::IpAddress) && a.client_ip != b.client_ip {
            changes |= KnownVariable::IpAddress;
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
    const OPT: u8 = OPT;

    fn negotiate<W: fmt::Write>(self, mut f: W, config: &TransformerConfig) -> fmt::Result {
        f.write_str(self.prefix)?;
        for variable in self.inner {
            variable.negotiate(&mut f, config)?;
        }
        Ok(())
    }
}
