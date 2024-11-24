pub(crate) mod charset;
pub use charset::CODE as CHARSET;

pub(crate) mod mccp;
pub use mccp::CODE_V1 as MCCP1;
pub use mccp::CODE_V2 as MCCP2;

pub(crate) mod mtts;
pub use mtts::CODE as MTTS;

pub(crate) mod naws;
pub use naws::CODE as NAWS;

/// Aardwolf Protocol
///
/// https://www.aardwolf.com/blog/2008/07/10/telnet-negotiation-control-mud-client-interaction/
pub const AARDWOLF: u8 = 102;

/// Achaea Telnet Client Protocol
///
/// http://www.ironrealms.com/rapture/manual/files/FeatATCP-txt.html
pub const ATCP: u8 = 200;

/// ECHO
pub const ECHO: u8 = 1;

/// Generic Mud Communication Protocol
///
/// https://tintin.mudhalla.net/protocols/gmcp/
pub const GMCP: u8 = 201;

/// MUD Sound Protocol
///
/// https://www.zuggsoft.com/zmud/msp.htm
pub const MSP: u8 = 90;

/// MUD Extension Protocol
///
/// https://www.zuggsoft.com/zmud/mxp.htm
pub const MXP: u8 = 91;

/// MUD-specific negotiations
pub const MUD_SPECIFIC: u8 = 102;

/// Suppress Go-Ahead
///
/// https://datatracker.ietf.org/doc/html/rfc858
pub const SGA: u8 = 3;

/// Zenith MUD Protocol
///
/// http://zmp.sourcemud.org/spec.shtml
pub const ZMP: u8 = 93;
