use std::io::Write;

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
pub(crate) struct Negotiator {
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

impl Negotiate for Negotiator {
    const CODE: u8 = CODE;

    fn negotiate(self, buf: &mut Vec<u8>, config: &TransformerConfig) {
        buf.push(0);
        match self.sequence {
            0 => buf.extend_from_slice(config.terminal_identification.as_bytes()),
            1 => buf.extend_from_slice(b"ANSI"),
            _ => write!(buf, "{}", bitmask(config)).unwrap(),
        }
    }
}
