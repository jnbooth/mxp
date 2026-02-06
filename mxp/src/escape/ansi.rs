/// Null character
pub const NULL: u8 = 0x00;
/// Start of Header
pub const SOH: u8 = 0x01;
/// Start of Text
pub const STX: u8 = 0x02;
/// End of Text
pub const ETX: u8 = 0x03;
/// End of Transmission
pub const EOT: u8 = 0x04;
/// Enquiry
pub const ENQ: u8 = 0x05;
/// Acknowledgement
pub const ACK: u8 = 0x06;
/// Ring the terminal bell
pub const BEL: u8 = 0x07;
/// Backspace
pub const BS: u8 = 0x08;
/// Horizontal tab
pub const HT: u8 = b'\t';
/// Linefeed
pub const LF: u8 = b'\n';
/// Vertical tab
pub const VT: u8 = 0x0B;
/// Form feed
pub const FF: u8 = 0x0C;
/// Carriage return
pub const CR: u8 = b'\r';
/// Shift in
pub const SO: u8 = 0x0E;
/// Shift out
pub const SI: u8 = 0x0F;
/// Data link escape
pub const DLE: u8 = 0x10;
/// Device control 1
pub const DC1: u8 = 0x11;
/// Device control 2
pub const DC2: u8 = 0x12;
/// Device control 3
pub const DC3: u8 = 0x13;
/// Device control 4
pub const DC4: u8 = 0x14;
/// Negative acknowledgement
pub const NAK: u8 = 0x15;
/// Synchronous idle
pub const SYN: u8 = 0x16;
/// End of transmission block
pub const ETB: u8 = 0x17;
/// Cancel
pub const CAN: u8 = 0x18;
/// End of medium
pub const EM: u8 = 0x19;
/// Substitute
pub const SUB: u8 = 0x1a;
/// Escape
pub const ESC: u8 = 0x1B;
/// File separator
pub const FS: u8 = 0x1C;
/// Group separator
pub const GS: u8 = 0x1D;
/// Record separator
pub const RS: u8 = 0x1E;
/// Unit separator
pub const US: u8 = 0x1F;
/// Delete character
pub const DEL: u8 = 0x7F;

/// Device Control String
pub const ESC_DCS: u8 = b'P';
/// Device Control String
pub const DCS: &str = "\x1BP";

/// Start of String
pub const ESC_SOS: u8 = b'X';
/// Start of String
pub const SOS: &str = "\x1BX";

/// Control Sequence Introducer
pub const ESC_CSI: u8 = b'[';
/// Control Sequence Introducer
pub const CSI: &str = "\x1B[";

/// String Terminator
pub const ESC_ST: u8 = b'\\';
/// String Terminator
pub const ST: &str = "\x1B\\";

/// Operating System Command
pub const ESC_OSC: u8 = b']';
/// Operating System Command
pub const OSC: &str = "\x1B]";

/// Privacy Message
pub const ESC_PM: u8 = b'^';
/// Privacy Message
pub const PM: &str = "\x1B^";

/// Application Program Command
pub const ESC_APC: u8 = b'_';
/// Application Program Command
pub const APC: &str = "\x1B_";

// Modes

pub const RESET: u8 = 0;
pub const BOLD: u8 = 1;
pub const FAINT: u8 = 2;
pub const ITALIC: u8 = 3;
pub const UNDERLINE: u8 = 4;
pub const SLOW_BLINK: u8 = 5;
pub const RAPID_BLINK: u8 = 6;
pub const INVERSE: u8 = 7;
pub const CONCEAL: u8 = 8;
pub const STRIKEOUT: u8 = 9;

pub const PRIMARY_FONT: u8 = 10;
pub const BLACKLETTER: u8 = 20;

pub const DOUBLE_UNDERLINE: u8 = 21;

/// Cancels [`BOLD`] and [`FAINT`].
pub const CANCEL_BOLD: u8 = 22;
/// Cancels [`ITALIC`] and [`BLACKLETTER`].
pub const CANCEL_ITALIC: u8 = 23;
/// Cancels [`UNDERLINE`].
pub const CANCEL_UNDERLINE: u8 = 24;
/// Cancels [`SLOW_BLINK`] and [`RAPID_BLINK`].
pub const CANCEL_BLINK: u8 = 25;
pub const PROPORTIONAL_SPACING: u8 = 26;
/// Cancels [`INVERSE`].
pub const CANCEL_INVERSE: u8 = 27;
/// Cancels [`CONCEAL`].
pub const CANCEL_CONCEAL: u8 = 28;
/// Cancels [`STRIKEOUT`].
pub const CANCEL_STRIKEOUT: u8 = 29;

pub const FG_BLACK: u8 = 30;
pub const FG_RED: u8 = 31;
pub const FG_GREEN: u8 = 32;
pub const FG_YELLOW: u8 = 33;
pub const FG_BLUE: u8 = 34;
pub const FG_MAGENTA: u8 = 35;
pub const FG_CYAN: u8 = 36;
pub const FG_WHITE: u8 = 37;
pub const FG_256_COLOR: u8 = 38;
pub const FG_DEFAULT: u8 = 39;

pub const BG_BLACK: u8 = 40;
pub const BG_RED: u8 = 41;
pub const BG_GREEN: u8 = 42;
pub const BG_YELLOW: u8 = 43;
pub const BG_BLUE: u8 = 44;
pub const BG_MAGENTA: u8 = 45;
pub const BG_CYAN: u8 = 46;
pub const BG_WHITE: u8 = 47;
pub const BG_256_COLOR: u8 = 48;
pub const BG_DEFAULT: u8 = 49;

/// Cancels [`PROPORTIONAL_SPACING`].
pub const CANCEL_PROPORTIONAL_SPACING: u8 = 50;
pub const FRAMED: u8 = 51;
pub const ENCIRCLED: u8 = 52;
pub const OVERLINED: u8 = 53;
/// Cancels [`FRAMED`] and [`ENCIRCLED`].
pub const CANCEL_FRAMED: u8 = 54;
/// Cancels [`OVERLINED`].
pub const CANCEL_OVERLINED: u8 = 55;

pub const UNDERLINE_COLOR: u8 = 58;
pub const UNDERLINE_COLOR_DEFAULT: u8 = 59;

pub const SUPERSCRIPT: u8 = 73;
pub const SUBSCRIPT: u8 = 74;
/// Cancels [`SUPERSCRIPT`] and [`SUBSCRIPT`].
pub const CANCEL_POSITION: u8 = 75;

pub const BEGIN_XTERM_COLOR: u8 = 5;
pub const BEGIN_TRUECOLOR: u8 = 2;
