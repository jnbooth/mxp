use std::error::Error;
use std::fmt;
use std::io::{self, Write};
use std::str::Utf8Error;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::escape::telnet;

/// Generic Mud Communication Protocol
///
/// https://tintin.mudhalla.net/protocols/gmcp/
pub const OPT: u8 = 201;

/// See [`Message::decode`].
pub fn decode(bytes: &[u8]) -> Result<Message<&str>, DecodeError> {
    Message::decode(bytes)
}

/// Writes the subnegotiation command to a writer, including IAC prefix and suffix.
pub fn encode_command<W: Write>(mut writer: W, command: &str) -> io::Result<()> {
    writer.write_all(&[telnet::IAC, telnet::SB, OPT])?;
    writer.write_all(command.as_bytes())?;
    writer.write_all(&[telnet::IAC, telnet::SE])
}

/// Writes the subnegotiation command and data to a writer, including IAC prefix and suffix.
/// Data is encoded as JSON.
#[cfg(feature = "json")]
pub fn encode<T, W>(mut writer: W, command: &str, value: &T) -> io::Result<()>
where
    T: Serialize,
    W: Write,
{
    writer.write_all(&[telnet::IAC, telnet::SB, OPT])?;
    writer.write_all(command.as_bytes())?;
    writer.write_all(b" ")?;
    serde_json::to_writer(&mut writer, value)?;
    writer.write_all(&[telnet::IAC, telnet::SE])
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    EmptyString,
    InvalidUtf8(Utf8Error),
}

impl From<Utf8Error> for DecodeError {
    fn from(value: Utf8Error) -> Self {
        Self::InvalidUtf8(value)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyString => f.write_str("received empty string"),
            Self::InvalidUtf8(e) => write!(f, "invalid UTF-8: {e}"),
        }
    }
}

impl Error for DecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EmptyString => None,
            Self::InvalidUtf8(e) => Some(e),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Message<S> {
    pub command: S,
    pub data: Option<S>,
}

impl<'a> Message<&'a str> {
    pub fn decode(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        fn split_data_from_command(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
            let pos = bytes.iter().position(|&c| c == b' ' || c == b'\n')?;
            let (command, data) = bytes.split_at(pos);
            let trimmed = data.trim_ascii();
            if trimmed.is_empty() {
                return None;
            }
            Some((command, trimmed))
        }

        if bytes.is_empty() {
            return Err(DecodeError::EmptyString);
        }
        let (command, data) = match split_data_from_command(bytes) {
            Some((command, data)) => (command, Some(str::from_utf8(data)?)),
            None => (bytes, None),
        };
        Ok(Self {
            command: str::from_utf8(command)?,
            data,
        })
    }

    #[cfg(feature = "json")]
    pub fn deserialize<'de, T: Deserialize<'de>>(&'de self) -> serde_json::Result<T> {
        serde_json::from_str(self.data.unwrap_or_default())
    }
}

impl<S: AsRef<[u8]>> Message<S> {
    pub fn encode<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_all(self.command.as_ref())?;
        if let Some(data) = &self.data {
            writer.write_all(b" ")?;
            writer.write_all(data.as_ref())?;
        }
        Ok(())
    }
}

#[cfg(feature = "json")]
impl Message<String> {
    pub fn serialize<T: Serialize>(command: String, data: &T) -> serde_json::Result<Self> {
        Ok(Self {
            command,
            data: Some(serde_json::to_string(data)?),
        })
    }
}
