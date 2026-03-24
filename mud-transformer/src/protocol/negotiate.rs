use std::fmt;
use std::ops::Not;

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
