use bytes::Bytes;

use super::OutputFragment;
use crate::protocol::msdp::MsdpValue;
use crate::protocol::negotiate::{TelnetSource, TelnetVerb};

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
