use std::io::Write;
use std::ops::Not;
use std::{fmt, io};

use crate::TransformerConfig;
use crate::escape::telnet;

pub(crate) trait Negotiate {
    const OPT: u8;

    fn negotiate<W: fmt::Write>(self, f: W, config: &TransformerConfig) -> fmt::Result;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TelnetSource {
    Client,
    Server,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TelnetVerb {
    Will = telnet::WILL as _,
    Wont = telnet::WONT as _,
    Do = telnet::DO as _,
    Dont = telnet::DONT as _,
}

impl Not for TelnetVerb {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Will => Self::Wont,
            Self::Wont => Self::Will,
            Self::Do => Self::Dont,
            Self::Dont => Self::Do,
        }
    }
}

impl fmt::Display for TelnetVerb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Will => "WILL".fmt(f),
            Self::Wont => "WONT".fmt(f),
            Self::Do => "DO".fmt(f),
            Self::Dont => "DONT".fmt(f),
        }
    }
}

pub(crate) fn write_escaping_iac<W: Write>(mut writer: W, bytes: &[u8]) -> io::Result<()> {
    let mut escaping = false;
    for slice in bytes.split(|&c| c == telnet::IAC) {
        if escaping {
            writer.write_all(&[telnet::IAC, telnet::IAC])?;
        } else {
            writer.write_all(&[telnet::IAC])?;
            escaping = true;
        }
        writer.write_all(slice)?;
    }
    Ok(())
}
