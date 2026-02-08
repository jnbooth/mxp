use bytes::Bytes;
use mxp::escape::telnet;

use super::OutputFragment;
use crate::protocol::msdp::MsdpValue;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TelnetSource {
    Client,
    Server,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TelnetVerb {
    Will = telnet::WILL as isize,
    Wont = telnet::WONT as isize,
    Do = telnet::DO as isize,
    Dont = telnet::DONT as isize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TelnetFragment {
    GoAhead,
    Msdp {
        name: Bytes,
        value: MsdpValue,
    },
    Mxp {
        enabled: bool,
    },
    Naws,
    Negotiation {
        source: TelnetSource,
        verb: TelnetVerb,
        code: u8,
    },
    ServerStatus {
        variable: Bytes,
        value: Bytes,
    },
    SetEcho {
        should_echo: bool,
    },
    Subnegotiation {
        code: u8,
        data: Bytes,
    },
}

impl From<TelnetFragment> for OutputFragment {
    fn from(value: TelnetFragment) -> Self {
        Self::Telnet(value)
    }
}
