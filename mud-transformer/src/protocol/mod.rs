pub(crate) mod ansi;

pub(crate) mod charset;
pub use charset::OPT as CHARSET;

pub mod gmcp;
pub use gmcp::OPT as GMCP;

pub(crate) mod mccp;
pub use mccp::OPT as MCCP2;

pub mod mnes;
pub use mnes::OPT as MNES;

pub mod msdp;
pub use msdp::OPT as MSDP;

pub mod mssp;
pub use mssp::OPT as MSSP;

pub mod mtts;
pub use mtts::OPT as MTTS;

pub(crate) mod naws;
pub use naws::OPT as NAWS;

pub(crate) mod negotiate;

pub(crate) mod xterm;

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

/// MUD Sound Protocol
///
/// https://www.zuggsoft.com/zmud/msp.htm
pub const MSP: u8 = 90;

/// MUD Extension Protocol
///
/// https://www.zuggsoft.com/zmud/mxp.htm
pub const MXP: u8 = 91;

/// Suppress Go-Ahead
///
/// https://datatracker.ietf.org/doc/html/rfc858
pub const SGA: u8 = 3;

/// Zenith MUD Protocol
///
/// http://zmp.sourcemud.org/spec.shtml
pub const ZMP: u8 = 93;
