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
/// > NOP if received, although IAC EOR should not normally be sent in this mode.
/// >
/// > As the EOR code indicates the end of an effective data unit, Telnet should attempt to send
/// > the data up to and including the EOR code together to promote communication efficiency.
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

pub const fn supports_do(code: u8, supports: bool) -> [u8; 3] {
    [IAC, if supports { DO } else { DONT }, code]
}

pub const fn supports_will(code: u8, supports: bool) -> [u8; 3] {
    [IAC, if supports { WILL } else { WONT }, code]
}

#[cfg(test)]
mod tests {
    use super::*;
    const CODE: u8 = 10;

    #[test]
    fn formats_do() {
        assert_eq!(supports_do(CODE, true), [IAC, DO, CODE]);
    }

    #[test]
    fn formats_dont() {
        assert_eq!(supports_do(CODE, false), [IAC, DONT, CODE]);
    }

    #[test]
    fn formats_will() {
        assert_eq!(supports_will(CODE, true), [IAC, WILL, CODE]);
    }

    #[test]
    fn formats_wont() {
        assert_eq!(supports_will(CODE, false), [IAC, WONT, CODE]);
    }
}
