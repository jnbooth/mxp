pub const ESC: u8 = 0x1B;
/// IAC WILL END-OF-RECORD.
///
/// Specified in [RFC 885](https://datatracker.ietf.org/doc/html/rfc885):
/// > The sender of this command requests permission to begin transmission of the Telnet
/// > END-OF-RECORD (EOR) code when transmitting data characters, or the sender of this command
/// > confirms it will now begin transmission of EORs with transmitted data characters.
pub const WILL_EOR: u8 = 0x19;
/// IAC END-OF-RECORD.
///
/// Specified in [RFC 885](https://datatracker.ietf.org/doc/html/rfc885):
/// > When the END-OF-RECORD option is in effect on the connection between a sender of data and
/// > the receiver of the data, the sender transmits EORs.
/// >
/// > When the END-OF-RECORD option is not in effect, the IAC EOR command should be treated as a
/// NOP if received, although IAC EOR should not normally be sent in this mode.
/// >
/// > As the EOR code indicates the end of an effective data unit, Telnet should attempt to send
/// > the data up to and including the EOR code together to promote communication efficiency.
/// see RFC 885
pub const EOR: u8 = 0xEF;

/// end of subnegotiation
pub const SE: u8 = 0xF0;
/// no operation
pub const NOP: u8 = 0xF1;
/// DataMark, see RFC 854
pub const DM: u8 = 0xF2;
/// Break
pub const BRK: u8 = 0xF3;
/// Interrupt Process
pub const IP: u8 = 0xF4;
/// Abort Output
pub const AO: u8 = 0xF5;
/// Are You There
pub const AYT: u8 = 0xF6;
/// Erase Character
pub const EC: u8 = 0xF7;
/// Erase Line
pub const EL: u8 = 0xF8;
/// Go Ahead
pub const GA: u8 = 0xF9;
/// subnegotiation
pub const SB: u8 = 0xFA;
pub const WILL: u8 = 0xFB;
pub const WONT: u8 = 0xFC;
pub const DO: u8 = 0xFD;
pub const DONT: u8 = 0xFE;
pub const IAC: u8 = 0xFF;

// Capability escape sequences
pub const ECHO: u8 = 0x01;
/// Negotiate About Window Size
pub const NAWS: u8 = 0x1F;
/// Negotiate About Character Set
pub const CHARSET: u8 = 0x2A;
/// want to know terminal type
pub const TERMINAL_TYPE: u8 = 0x18;
/// telnet negotiation code for starting compression v1
pub const COMPRESS: u8 = 0x55;
/// telnet negotiation code for starting compression v2
pub const COMPRESS2: u8 = 0x56;
/// telnet negotiation code MUD-specific negotiations
pub const MUD_SPECIFIC: u8 = 0x66;
/// suppress go-ahead
pub const SGA: u8 = 0x03;
/// telnet negotiation code for MUD Sound Protocol (MSP)
pub const MSP: u8 = 0x5A;
/// telnet negotiation code for MUD Extension Protocol (MXP)
pub const MXP: u8 = 0x5B;
/// http://zmp.sourcemud.org/spec.shtml
pub const ZMP: u8 = 0x5D;
/// https://www.aardwolf.com/blog/2008/07/10/telnet-negotiation-control-mud-client-interaction/
pub const AARDWOLF: u8 = 0x66;
/// http://www.ironrealms.com/rapture/manual/files/FeatATCP-txt.html
pub const ATCP: u8 = 0xC8;
/// https://tintin.mudhalla.net/protocols/gmcp/
pub const GMCP: u8 = 0xC9;
/// reserved
pub const EXT: u8 = 0xFF;

// Subnegotiation escape sequences
pub const TTYPE_IS: u8 = 0x00;
pub const TTYPE_SEND: u8 = 0x01;
pub const ACCEPT: u8 = 0x02;
pub const REJECT: u8 = 0x03;

// Sequences
pub const TTYPE_PREFIX: &[u8] = &[IAC, SB, TERMINAL_TYPE, TTYPE_IS];
pub const TTYPE_SUFFIX: &[u8] = &[IAC, SE];

macro_rules! charset {
        ($($b:literal),*) => (&[IAC, SB, CHARSET, ACCEPT, $($b,)* IAC, SE])
    }
const CHARSET_UTF8: &[u8] = charset!(b'U', b'T', b'F', b'-', b'8');
const CHARSET_US_ASCII: &[u8] = charset!(b'U', b'S', b'-', b'A', b'S', b'C', b'I', b'I');
const REJECT_SUBNEGOTIATION: &[u8] = &[IAC, SB, REJECT, IAC, SE];

pub const fn escape_char(s: u8) -> Option<&'static str> {
    Some(match s {
        self::ESC => "[ESC]",
        self::WILL_EOR => "[WILL_EOR]",
        self::EOR => "[EOR]",
        self::SE => "[SE]",
        self::NOP => "[NOP]",
        self::DM => "[DM]",
        self::BRK => "[BRK]",
        self::IP => "[IP]",
        self::AO => "[AO]",
        self::AYT => "[AYT]",
        self::EC => "[EC]",
        self::EL => "[EL]",
        self::GA => "[GA]",
        self::SB => "[SB]",
        self::WILL => "[WILL]",
        self::WONT => "[WONT]",
        self::DO => "[DO]",
        self::DONT => "[DONT]",
        self::IAC => "[IAC]",

        self::ECHO => "[ECHO]/[TTYPE_SEND]",
        self::NAWS => "[NAWS]",
        self::CHARSET => "[CHARSET]",
        self::TERMINAL_TYPE => "[TT]",
        self::COMPRESS => "[COMPRESS]",
        self::COMPRESS2 => "[COMPRESS2]",
        self::MUD_SPECIFIC => "[MUDSPECIFIC]",
        self::SGA => "[SGA]/[REJECT]",
        self::MSP => "[MSP]",
        self::MXP => "[MXP]",
        self::ZMP => "[ZMP]",
        self::ATCP => "[ATCP]",
        self::GMCP => "[GMCP]",
        self::TTYPE_IS => "[TTYPE_IS]",
        self::ACCEPT => "[ACCEPT]",
        _ => return None,
    })
}

pub fn escape(s: &[u8]) -> Vec<u8> {
    let mut escaped = Vec::with_capacity(s.len());
    for &c in s {
        match escape_char(c) {
            None => escaped.push(c),
            Some(esc) => escaped.extend_from_slice(esc.as_bytes()),
        }
    }
    escaped
}

pub fn find_charset(data: &[u8], utf8: bool) -> &'static [u8] {
    let delim = data[1];
    let mut fragments = data[2..].split(|&c| c == delim);
    if !utf8 {
        return if fragments.any(|x| x == b"US-ASCII") {
            CHARSET_US_ASCII
        } else {
            REJECT_SUBNEGOTIATION
        };
    };
    let mut supports_ascii = false;
    for fragment in fragments {
        if fragment == b"UTF-8" {
            return CHARSET_UTF8;
        }
        if fragment == b"US-ASCII" {
            supports_ascii = true;
        }
    }
    if supports_ascii {
        CHARSET_US_ASCII
    } else {
        REJECT_SUBNEGOTIATION
    }
}

pub const fn supports_do(code: u8, supports: bool) -> [u8; 3] {
    [IAC, if supports { DO } else { DONT }, code]
}

pub const fn supports_will(code: u8, supports: bool) -> [u8; 3] {
    [IAC, if supports { WILL } else { WONT }, code]
}
