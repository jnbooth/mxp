#[cfg(feature = "json")]
use std::io::{self, Write};

#[cfg(feature = "json")]
use crate::escape::telnet;

/// Generic Mud Communication Protocol
///
/// https://tintin.mudhalla.net/protocols/gmcp/
pub const OPT: u8 = 201;

#[cfg(feature = "json")]
pub use serde_json::from_slice as decode;

#[cfg(feature = "json")]
pub fn encode<T: serde::Serialize, W: Write>(mut writer: W, value: &T) -> io::Result<()> {
    writer.write_all(&[telnet::IAC, telnet::SB, OPT])?;
    serde_json::to_writer(&mut writer, value)?;
    writer.write_all(&[telnet::IAC, telnet::SE])
}
