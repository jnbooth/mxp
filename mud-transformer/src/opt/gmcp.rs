use std::error::Error;
use std::fmt;
use std::io::{self, Write};
use std::str::Utf8Error;

use bytes::Bytes;
use bytestring::ByteString;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::bytestring_ext::ByteStringExt;
use crate::escape::telnet;

/// Generic Mud Communication Protocol
///
/// https://tintin.mudhalla.net/protocols/gmcp/
pub const OPT: u8 = 201;

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
pub struct Message {
    pub command: ByteString,
    pub data: Option<ByteString>,
}

impl Message {
    pub fn decode(mut bytes: Bytes) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError::EmptyString);
        }
        let data = match Self::split_data_from_command(&mut bytes) {
            Some(data) => Some(ByteString::from_utf8(data)?),
            None => None,
        };
        Ok(Self {
            command: ByteString::from_utf8(bytes)?,
            data,
        })
    }

    pub fn encode<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_all((*self.command).as_bytes())?;
        if let Some(data) = &self.data {
            writer.write_all(b" ")?;
            writer.write_all(str::as_bytes(data))?;
        }
        Ok(())
    }

    #[cfg(feature = "json")]
    pub fn deserialize<'de, T: Deserialize<'de>>(&'de self) -> serde_json::Result<T> {
        serde_json::from_str(self.data.as_deref().unwrap_or_default())
    }

    #[cfg(feature = "json")]
    pub fn serialize<T: Serialize>(command: String, data: &T) -> serde_json::Result<Self> {
        Ok(Self {
            command: command.into(),
            data: Some(serde_json::to_string(data)?.into()),
        })
    }

    fn split_data_from_command(bytes: &mut Bytes) -> Option<Bytes> {
        let pos = bytes.iter().position(|&c| c == b' ' || c == b'\n')?;
        let data = bytes.split_off(pos);
        let trimmed = data.trim_ascii();
        if trimmed.is_empty() {
            return None;
        }
        Some(data.slice_ref(trimmed))
    }
}

pub fn decode(bytes: Bytes) -> Result<Message, DecodeError> {
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
