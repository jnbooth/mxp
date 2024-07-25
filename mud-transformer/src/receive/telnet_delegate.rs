#![allow(unused_variables)]

pub trait TelnetDelegate {
    #[inline(always)]
    fn on_iac_ga(&mut self) {}

    #[inline(always)]
    fn on_telnet_option(&mut self, data: &[u8]) {}

    #[inline(always)]
    fn on_telnet_subnegotiation(&mut self, negotiation_type: u8, data: &[u8]) {}
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoopTelnetDelegate;

impl TelnetDelegate for NoopTelnetDelegate {}
