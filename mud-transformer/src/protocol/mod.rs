pub(crate) mod ansi;

pub(crate) mod negotiate;

pub(crate) mod xterm;

/// ECHO
pub const ECHO: u8 = 1;

/// Suppress Go-Ahead
///
/// https://datatracker.ietf.org/doc/html/rfc858
pub const SGA: u8 = 3;

pub mod status;
pub use status::OPT as STATUS; // 5

pub mod mtts;
pub use mtts::OPT as MTTS; // 24

/// IAC WILL END-OF-RECORD.
///
/// Specified in [RFC 885](https://datatracker.ietf.org/doc/html/rfc885):
/// > The sender of this command requests permission to begin transmission of the Telnet
/// > END-OF-RECORD (EOR) code when transmitting data characters, or the sender of this command
/// > confirms it will now begin transmission of EORs with transmitted data characters.
pub const EOR: u8 = 25;

pub mod naws;
pub use naws::OPT as NAWS; // 31

pub mod mnes;
pub use mnes::OPT as MNES; // 39

pub mod charset;
pub use charset::OPT as CHARSET; // 42

pub mod msdp;
pub use msdp::OPT as MSDP; // 69

pub mod mssp;
pub use mssp::OPT as MSSP; // 70

pub(crate) mod mccp;
pub use mccp::OPT as MCCP2; // 86

pub mod msp;
pub use msp::OPT as MSP; // 90

/// MUD Extension Protocol
///
/// https://www.zuggsoft.com/zmud/mxp.htm
pub const MXP: u8 = 91;

/// Zenith MUD Protocol
///
/// https://discworld.starturtle.net/external/protocols/zmp.html
pub const ZMP: u8 = 93;

/// Aardwolf Protocol
///
/// https://www.aardwolf.com/blog/2008/07/10/telnet-negotiation-control-mud-client-interaction/
pub const AARDWOLF: u8 = 102;

/// Achaea Telnet Client Protocol
///
/// http://www.ironrealms.com/rapture/manual/files/FeatATCP-txt.html
pub const ATCP: u8 = 200;

pub mod gmcp;
pub use gmcp::OPT as GMCP; // 201
