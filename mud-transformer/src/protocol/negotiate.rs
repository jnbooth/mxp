use std::fmt;

use mxp::escape::telnet;

use crate::TransformerConfig;

pub(crate) trait Negotiate {
    const CODE: u8;

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
