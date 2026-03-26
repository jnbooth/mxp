use std::io::{self, Write};

use crate::escape::telnet;

/// [RFC 1073](https://datatracker.ietf.org/doc/html/rfc1073): NAWS (Negotiate About Window Size)
pub const OPT: u8 = 31;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowSize {
    pub width: u16,
    pub height: u16,
}

impl WindowSize {
    pub const fn subnegotiation(self) -> [u8; 9] {
        let [width_high, width_low] = self.width.to_be_bytes();
        let [height_high, height_low] = self.height.to_be_bytes();
        [
            telnet::IAC,
            telnet::SB,
            OPT,
            width_high,
            width_low,
            height_high,
            height_low,
            telnet::IAC,
            telnet::SE,
        ]
    }

    pub fn encode<W: Write>(self, mut writer: W) -> io::Result<()> {
        writer.write_all(&self.subnegotiation())
    }
}

pub const fn subnegotiate(width: u16, height: u16) -> [u8; 9] {
    WindowSize { width, height }.subnegotiation()
}
