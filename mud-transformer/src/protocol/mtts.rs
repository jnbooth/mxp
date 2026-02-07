use std::fmt;

use super::Negotiate;
use crate::transformer::TransformerConfig;

/// MUD Terminal Type Standard
///
/// https://tintin.mudhalla.net/protocols/mtts/
pub const CODE: u8 = 24;

pub const SEND: u8 = 1;

#[allow(clippy::upper_case_acronyms)]
struct MTTS;

#[allow(unused)]
impl MTTS {
    /// Client supports all ANSI color codes. Supporting blink and underline is optional.
    pub const ANSI: u16 = 1;
    /// Client supports most VT100 codes.
    pub const VT100: u16 = 2;
    /// Client is using UTF-8 character encoding.
    pub const UTF_8: u16 = 4;
    /// Client supports all xterm 256 color codes (a.k.a. "256 COLORS").
    pub const XTERM_COLORS: u16 = 8;
    /// Client supports xterm mouse tracking.
    pub const MOUSE_TRACKING: u16 = 16;
    /// Client supports the OSC color palette.
    pub const OSC_COLOR_PALETTE: u16 = 32;
    /// Client is using a screen reader.
    pub const SCREEN_READER: u16 = 64;
    /// Client is a proxy allowing different users to connect from the same IP address.
    pub const PROXY: u16 = 128;
    /// Client supports truecolor codes using semicolon notation.
    pub const TRUECOLOR: u16 = 256;
    /// Client supports the Mud New Environment Standard for information exchange.
    pub const MNES: u16 = 512;
    /// Client supports the Mud Server Link Protocol for clickable link handling.
    pub const MSLP: u16 = 1024;
    /// Client supports SSL for data encryption, preferably TLS 1.3 or higher.
    pub const SSL: u16 = 2048;
}

pub const fn ttype(config: &TransformerConfig) -> &'static str {
    if !config.console_controls {
        return "ANSI-TRUECOLOR";
    }
    if !config.mouse_tracking || !config.window_controls {
        return "VT100-TRUECOLOR";
    }
    "XTERM-TRUECOLOR"
}

pub const fn bitmask(config: &TransformerConfig) -> u16 {
    const fn mask(enable: bool, capability: u16) -> u16 {
        if enable { capability } else { 0 }
    }

    MTTS::ANSI
        | mask(config.console_controls, MTTS::VT100)
        | mask(!config.disable_utf8, MTTS::UTF_8)
        | MTTS::XTERM_COLORS
        | mask(config.mouse_tracking, MTTS::MOUSE_TRACKING)
        | MTTS::OSC_COLOR_PALETTE
        | mask(config.screen_reader, MTTS::SCREEN_READER)
        | mask(config.proxy, MTTS::PROXY)
        | MTTS::TRUECOLOR
        | MTTS::MNES
        | mask(config.ssl, MTTS::SSL)
}

#[derive(Copy, Clone, Debug)]
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

    fn negotiate<W: fmt::Write>(self, mut f: W, config: &TransformerConfig) -> fmt::Result {
        match self.sequence {
            0 => write!(f, "\0{}", &config.terminal_identification),
            1 => write!(f, "\0{}", ttype(config)),
            _ => write!(f, "\0{}", bitmask(config)),
        }
    }
}
