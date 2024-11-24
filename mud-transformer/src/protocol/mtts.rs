use std::fmt::{self, Display, Formatter};

use super::Negotiate;
use crate::transformer::TransformerConfig;

/// MUD Terminal Type Standard
///
/// https://tintin.mudhalla.net/protocols/mtts/
pub const CODE: u8 = 24;

pub const SEND: u8 = 1;

#[inline]
const fn mask(enable: bool, capability: u16) -> u16 {
    if enable {
        capability
    } else {
        0
    }
}

pub const fn bitmask(config: &TransformerConfig) -> u16 {
    1 // ANSI
            | 8 // 256 colors
            | 256 // true color
            | 512 // Mud New Environment Standard
            | mask(config.screen_reader, 64) // screen reader
            | mask(!config.disable_utf8, 4) // UTF-8
            | mask(config.ssl, 2048) // SSL
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Negotiator {
    sequence: u8,
}

impl Default for Negotiator {
    fn default() -> Self {
        Self::new()
    }
}

impl Negotiator {
    pub const fn new() -> Self {
        Self { sequence: 0 }
    }

    pub fn reset(&mut self) {
        self.sequence = 0;
    }

    pub fn advance(&mut self) {
        self.sequence = if self.sequence == 2 {
            0
        } else {
            self.sequence + 1
        };
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Subnegotiation<'a> {
    config: &'a TransformerConfig,
    sequence: u8,
}

impl<'a> Display for Subnegotiation<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("\x00")?;
        match self.sequence {
            0 => f.write_str(&self.config.terminal_identification),
            1 => f.write_str("ANSI"),
            _ => bitmask(self.config).fmt(f),
        }
    }
}

impl Negotiate for Negotiator {
    const CODE: u8 = CODE;

    type Output<'a> = Subnegotiation<'a>;

    fn negotiate(self, config: &TransformerConfig) -> Subnegotiation {
        Subnegotiation {
            config,
            sequence: self.sequence,
        }
    }
}
