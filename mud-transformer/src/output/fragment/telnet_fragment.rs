use bytes::Bytes;

use super::OutputFragment;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TelnetSource {
    Client,
    Server,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TelnetVerb {
    Do,
    Dont,
    Will,
    Wont,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TelnetFragment {
    GoAhead,
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
