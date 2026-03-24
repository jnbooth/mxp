use std::io::{self, Write};

use crate::escape::telnet;

/// Generic Mud Communication Protocol
///
/// https://tintin.mudhalla.net/protocols/gmcp/
pub const OPT: u8 = 201;

#[cfg(feature = "json")]
pub use serde_json::from_slice as decode;

pub fn encode_command<S, W>(mut writer: W, command: &S) -> io::Result<()>
where
    S: AsRef<[u8]>,
    W: Write,
{
    writer.write_all(&[telnet::IAC, telnet::SB, OPT])?;
    writer.write_all(command.as_ref())?;
    writer.write_all(&[telnet::IAC, telnet::SE])
}

#[cfg(feature = "json")]
pub fn encode<S, T, W>(mut writer: W, command: &S, value: &T) -> io::Result<()>
where
    S: AsRef<[u8]>,
    T: serde::Serialize,
    W: Write,
{
    writer.write_all(&[telnet::IAC, telnet::SB, OPT])?;
    writer.write_all(command.as_ref())?;
    writer.write_all(b" ")?;
    serde_json::to_writer(&mut writer, value)?;
    writer.write_all(&[telnet::IAC, telnet::SE])
}
