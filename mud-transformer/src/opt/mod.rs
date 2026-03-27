use std::fmt;

pub mod status;
pub use status::OPT as STATUS; // 5

pub mod mtts;
pub use mtts::OPT as MTTS; // 24

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

pub mod mccp2;
pub use mccp2::OPT as MCCP2; // 86

pub mod msp;
pub use msp::OPT as MSP; // 90

pub mod mxp;
pub use mxp::OPT as MXP; // 91

pub mod gmcp;
pub use gmcp::OPT as GMCP; // 201

/// [RFC 856](https://datatracker.ietf.org/doc/html/rfc856): Binary Transmission
pub const TRANSMIT_BINARY: u8 = 0;

/// [RFC 857](https://datatracker.ietf.org/doc/html/rfc857): Echo
pub const ECHO: u8 = 1;

/// [NIC 15391](https://web.mit.edu/krb5/doc.old/protocols/telnet/nic15391): Prepare to Reconnect
pub const RCP: u8 = 2;

/// [RFC 858](https://datatracker.ietf.org/doc/html/rfc858): Suppress Go Ahead
pub const SUPPRESS_GO_AHEAD: u8 = 3;

/// [NIC 15393](https://web.mit.edu/krb5/doc.old/protocols/telnet/nic15393): Negotiate Approximate Message Size
pub const NAMS: u8 = 4;

/// [RFC 860](https://datatracker.ietf.org/doc/html/rfc860): Timing Mark
pub const TIMING_MARK: u8 = 6;

/// [RFC 726](https://datatracker.ietf.org/doc/html/rfc726): Remote Controlled Transmssion and Echoing Telnet Option
pub const RCTE: u8 = 7;

/// [RFC 587](https://datatracker.ietf.org/doc/html/rfc587): Negotiate About Output Line Width
pub const NAOL: u8 = 8;

/// [RFC 587](https://datatracker.ietf.org/doc/html/rfc587): Negotiate About Output Page Size
pub const NAOP: u8 = 9;

/// [RFC 652](https://datatracker.ietf.org/doc/html/rfc652): Negotiate About Output Carriage-Return Disposition
pub const NAOCRD: u8 = 10;

/// [RFC 653](https://datatracker.ietf.org/doc/html/rfc653): Negotiate About Output Horizontal Tab Stop
pub const NAOHTS: u8 = 11;

/// [RFC 654](https://datatracker.ietf.org/doc/html/rfc654): Negotiate About Output Horizontal Tab Disposition
pub const NAOHTD: u8 = 12;

/// [RFC 655](https://datatracker.ietf.org/doc/html/rfc655): Negotiate About Output Formfeed Disposition
pub const NAOFFD: u8 = 13;

/// [RFC 656](https://datatracker.ietf.org/doc/html/rfc656): Negotiate About Vertical Tabstops
pub const NAOVTS: u8 = 14;

/// [RFC 657](https://datatracker.ietf.org/doc/html/rfc657): Negotiate About Output Vertical Tab Disposition
pub const NAOVTD: u8 = 15;

/// [RFC 658](https://datatracker.ietf.org/doc/html/rfc658): Negotiate About Output Linefeed Disposition
pub const NAOLFD: u8 = 16;

/// [RFC 698](https://datatracker.ietf.org/doc/html/rfc698): Extended ASCII
pub const EXTEND_ASCII: u8 = 17;

/// [RFC 727](https://datatracker.ietf.org/doc/html/rfc727): Logout
pub const LOGOUT: u8 = 18;

/// [RFC 735](https://datatracker.ietf.org/doc/html/rfc735): Byte Macro
pub const BM: u8 = 19;

/// [RFC 1043](https://datatracker.ietf.org/doc/html/rfc1043): Data Entry Terminal
pub const DET: u8 = 20;

/// [RFC 736](https://datatracker.ietf.org/doc/html/rfc736): SUPDUP
pub const SUPDUP: u8 = 21;

/// [RFC 749](https://datatracker.ietf.org/doc/html/rfc749): SUPDUP-OUTPUT
pub const SUPDUP_OUTPUT: u8 = 22;

/// [RFC 779](https://datatracker.ietf.org/doc/html/rfc779): Send-Location
pub const SEND_LOCATION: u8 = 23;

/// [RFC 1091](https://datatracker.ietf.org/doc/html/rfc1091): Terminal-Type
pub const TERMINAL_TYPE: u8 = 24;

/// [RFC 885](https://datatracker.ietf.org/doc/html/rfc885): End of Record
pub const END_OF_RECORD: u8 = 25;

/// [RFC 927](https://datatracker.ietf.org/doc/html/rfc927): TACACS User Identification
pub const TUID: u8 = 26;

/// [RFC 933](https://datatracker.ietf.org/doc/html/rfc933): Output Marking Telnet
pub const OUTMRK: u8 = 27;

/// [RFC 946](https://datatracker.ietf.org/doc/html/rfc946): Telnet Terminal Location Number
pub const TTYLOC: u8 = 28;

/// [RFC 1041](https://datatracker.ietf.org/doc/html/rfc1041): 3270 Regime
pub const REGIME_3270: u8 = 29;

/// [RFC 1053](https://datatracker.ietf.org/doc/html/rfc1053): X.3 Pad
pub const X3_PAD: u8 = 30;

/// [RFC 1079](https://datatracker.ietf.org/doc/html/rfc1079): Terminal Speed
pub const TERMINAL_SPEED: u8 = 32;

/// [RFC 1372](https://datatracker.ietf.org/doc/html/rfc1372): Remote Flow Control
pub const TOGGLE_FLOW_CONTROL: u8 = 33;

/// [RFC 1184](https://datatracker.ietf.org/doc/html/rfc1184): Linemode
pub const LINEMODE: u8 = 34;

/// [RFC 1096](https://datatracker.ietf.org/doc/html/rfc1096): X Display Location (XDISPLOC)
pub const X_DISPLAY_LOCATION: u8 = 35;

/// [RFC 1408](https://datatracker.ietf.org/doc/html/rfc1408): Environment
pub const ENVIRON: u8 = 36;

/// [RFC 2941](https://datatracker.ietf.org/doc/html/rfc2941): Authentication
pub const AUTHENTICATION: u8 = 37;

/// [RFC 1372](https://datatracker.ietf.org/doc/html/rfc2946): Data Encryption
pub const ENCRYPT: u8 = 38;

/// [RFC 1572](https://datatracker.ietf.org/doc/html/rfc1572): Environment
pub const NEW_ENVIRON: u8 = 39;

/// [RFC 2355](https://datatracker.ietf.org/doc/html/rfc2355): TN3270 Enhancements
pub const TN3270E: u8 = 40;

/// [IANA](https://www.iana.org/assignments/telnet-options/telnet-options.xhtml): XAUTH
pub const XAUTH: u8 = 41;

/// [IANA](https://www.iana.org/assignments/telnet-options/telnet-options.xhtml): Remote Serial Port
pub const RSP: u8 = 43;

/// [RFC 2217](https://datatracker.ietf.org/doc/html/rfc2217): Com Port Control
pub const COM_PORT_OPTION: u8 = 44;

/// [IANA](https://www.iana.org/assignments/telnet-options/telnet-options.xhtml): Suppress Local Echo
pub const SUPPRESS_LOCAL_ECHO: u8 = 45;

/// [IANA](https://www.iana.org/assignments/telnet-options/telnet-options.xhtml): Start TLS
pub const START_TLS: u8 = 46;

/// [RFC 2840](https://datatracker.ietf.org/doc/html/rfc2840): KERMIT
pub const KERMIT: u8 = 47;

/// [IANA](https://www.iana.org/assignments/telnet-options/telnet-options.xhtml): SEND-URL
pub const SEND_URL: u8 = 48;

/// [IANA](https://www.iana.org/assignments/telnet-options/telnet-options.xhtml): FORWARD_X
pub const FORWARD_X: u8 = 49;

/// MUD Client Compression Protocol
///
/// https://tintin.mudhalla.net/protocols/mccp/
pub const MCCP: u8 = 85;

/// MUD Client Compression Protocol v3
///
/// https://tintin.mudhalla.net/protocols/mccp/
pub const MCCP3: u8 = 87;

/// Zenith MUD Protocol
///
/// https://discworld.starturtle.net/external/protocols/zmp.html
pub const ZMP: u8 = 93;

/// Aardwolf Protocol
///
/// https://www.aardwolf.com/blog/2008/07/10/telnet-negotiation-control-mud-client-interaction/
pub const AARDWOLF: u8 = 102;

/// [IANA](https://www.iana.org/assignments/telnet-options/telnet-options.xhtml): PRAGMA LOGON
pub const PRAGMA_LOGON: u8 = 138;

/// [IANA](https://www.iana.org/assignments/telnet-options/telnet-options.xhtml): SSPI LOGON
pub const SSPI_LOGON: u8 = 139;

/// [IANA](https://www.iana.org/assignments/telnet-options/telnet-options.xhtml): PRAGMA HEARTBEAT
pub const PRAGMA_HEARTBEAT: u8 = 140;

/// Achaea Telnet Client Protocol
///
/// http://www.ironrealms.com/rapture/manual/files/FeatATCP-txt.html
pub const ATCP: u8 = 200;

/// [RFC 861](https://datatracker.ietf.org/doc/html/rfc861): Extended Options - List (EXTENDED-OPTIONS-LIST)
pub const EXOPL: u8 = 255;

pub const fn name(code: u8) -> Option<&'static str> {
    Some(match code {
        TRANSMIT_BINARY => "TRANSMIT-BINARY",
        ECHO => "ECHO",
        RCP => "RCP",
        SUPPRESS_GO_AHEAD => "SUPPRESS-GO-AHEAD",
        NAMS => "NAMS",
        STATUS => "STATUS",
        TIMING_MARK => "TIMING-MARK",
        RCTE => "RCTE",
        NAOL => "NAOL",
        NAOP => "NAOP",
        NAOCRD => "NAOCRD",
        NAOHTS => "NAOHTS",
        NAOHTD => "NAOHTD",
        NAOFFD => "NAOFFD",
        NAOVTS => "NAOVTS",
        NAOVTD => "NAOVTD",
        NAOLFD => "NAOLFD",
        EXTEND_ASCII => "EXTEND-ASCII",
        LOGOUT => "LOGOUT",
        BM => "BM",
        DET => "DET",
        SUPDUP => "SUPDUP",
        SUPDUP_OUTPUT => "SUPDUP-OUTPUT",
        SEND_LOCATION => "SEND-LOCATION",
        TERMINAL_TYPE => "TERMINAL-TYPE",
        END_OF_RECORD => "END-OF-RECORD",
        TUID => "TUID",
        OUTMRK => "OUTMRK",
        TTYLOC => "TTYLOC",
        REGIME_3270 => "3270-REGIME",
        X3_PAD => "X.3-PAD",
        NAWS => "NAWS",
        TERMINAL_SPEED => "TERMINAL-SPEED",
        TOGGLE_FLOW_CONTROL => "TOGGLE-FLOW-CONTROL",
        LINEMODE => "LINEMODE",
        X_DISPLAY_LOCATION => "X-DISPLAY-LOCATION",
        ENVIRON => "ENVIRON",
        AUTHENTICATION => "AUTHENTICATION",
        ENCRYPT => "ENCRYPT",
        NEW_ENVIRON => "NEW-ENVIRON",
        TN3270E => "TN3270E",
        XAUTH => "XAUTH",
        CHARSET => "CHARSET",
        RSP => "RSP",
        COM_PORT_OPTION => "COM-PORT-OPTION",
        SUPPRESS_LOCAL_ECHO => "SUPPRESS-LOCAL-ECHO",
        START_TLS => "START-TLS",
        KERMIT => "KERMIT",
        SEND_URL => "SEND-URL",
        FORWARD_X => "FORWARD_X",
        MSDP => "MSDP",
        MSSP => "MSSP",
        MCCP => "MCCP",
        MCCP2 => "MCCP2",
        MCCP3 => "MCCP3",
        MSP => "MSP",
        MXP => "MXP",
        ZMP => "ZMP",
        AARDWOLF => "AARDWOLF",
        PRAGMA_LOGON => "PRAGMA-LOGON",
        SSPI_LOGON => "SSPI-LOGON",
        PRAGMA_HEARTBEAT => "PRAGMA-HEARTBEAT",
        ATCP => "ATCP",
        GMCP => "GMCP",
        EXOPL => "EXOPL",
        _ => return None,
    })
}

pub const fn display(code: u8) -> OptDisplay {
    OptDisplay(code)
}

#[doc(hidden)]
pub struct OptDisplay(u8);

impl fmt::Display for OptDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = self.0;
        if f.alternate() {
            let name = name(code).unwrap_or("Unknown");
            write!(f, "{name} ({code})")
        } else {
            match name(code) {
                Some(name) => name.fmt(f),
                None => code.fmt(f),
            }
        }
    }
}
