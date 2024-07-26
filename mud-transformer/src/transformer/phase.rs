use enumeration::Enum;

use mxp::escape::telnet;

const fn is_phase_reset_character(c: u8) -> bool {
    matches!(c, b'\r' | b'\n' | telnet::ESC | telnet::IAC)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Phase {
    /// Normal text
    Normal,
    /// Received an escape
    Esc,
    /// Processing an ANSI escape sequence
    DoingCode,
    /// Received TELNET IAC (interpret as command)
    Iac,
    /// Received TELNET WILL
    Will,
    /// Received TELNET WONT
    Wont,
    /// Received TELNET DO
    Do,
    /// Received TELNET DONT
    Dont,
    /// Received TELNET IAC SB
    Sb,
    /// Received TELNET IAC SB c (collecting data, awaiting IAC SE)
    Subnegotiation,
    /// Received TELNET IAC SB c <data> IAC (awaiting IAC or SE)
    SubnegotiationIac,
    /// Received TELNET IAC COMPRESS
    Compress,
    /// Received TELNET IAC COMPRESS WILL
    CompressWill,

    // see: https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit
    /// Received ESC[38;
    Foreground256Start,
    /// Received ESC[38;5;
    Foreground256Finish,
    /// Received ESC[48;
    Background256Start,
    /// Received ESC[48;5;
    Background256Finish,

    // see: https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit
    /// Received ESC[38;2;
    Foreground24bFinish,
    /// Received ESC[38;2;r;
    Foreground24brFinish,
    /// Received ESC[38;2;r;g;
    Foreground24bgFinish,
    /// Received ESC[38;2;r;g;b;
    Foreground24bbFinish,
    /// Received ESC[48;2;
    Background24bFinish,
    /// Received ESC[48;2;r;
    Background24brFinish,
    /// Received ESC[48;2;r;g;
    Background24bgFinish,
    /// Received ESC[48;2;r;g;b
    Background24bbFinish,

    /// Received 110 xxxxx, 1110 xxxx, or 11110 xxx
    Utf8Character,

    // MXP modes
    /// Collecting element, eg. < xxxxx >. Starts on <, stops on >
    MxpElement,
    /// Collecting comment, eg. <!-- xxxxx -->. Starts on <!--, stops on -->
    MxpComment,
    /// Collecting quote inside element, eg. <color='red'>
    MxpQuote,
    /// Collecting entity, eg. &gt; . Starts on &, stops on ;
    MxpEntity,
    /// Text sent from the server at the beginning of a session.
    MxpWelcome,
}

impl Default for Phase {
    fn default() -> Self {
        Self::Normal
    }
}

impl Phase {
    pub const fn is_mxp(self) -> bool {
        matches!(
            self,
            Self::MxpElement
                | Self::MxpComment
                | Self::MxpQuote
                | Self::MxpEntity
                | Self::MxpWelcome
        )
    }

    pub const fn is_mxp_mode_change(self) -> bool {
        matches!(self, Self::MxpWelcome)
    }

    pub const fn is_phase_reset(self, c: u8) -> bool {
        is_phase_reset_character(c) && !self.is_iac(c) && !self.is_subnegotiation()
    }

    const fn is_subnegotiation(self) -> bool {
        matches!(
            self,
            Self::Sb | Self::Subnegotiation | Self::SubnegotiationIac
        )
    }

    const fn is_iac(self, c: u8) -> bool {
        c == telnet::IAC && matches!(self, Self::Iac)
    }
}
