use mxp::escape::telnet;

/// Negotiate About Window Size
///
/// https://datatracker.ietf.org/doc/html/rfc1073
pub const CODE: u8 = 31;

pub const fn subnegotiate(width: u16, height: u16) -> [u8; 9] {
    let [width_high, width_low] = width.to_be_bytes();
    let [height_high, height_low] = height.to_be_bytes();
    [
        telnet::IAC,
        telnet::SB,
        CODE,
        width_high,
        width_low,
        height_high,
        height_low,
        telnet::IAC,
        telnet::SE,
    ]
}
