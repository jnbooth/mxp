use mxp::escape::{ansi, telnet};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Normal text
    #[default]
    Normal,
    /// Received an escape
    Esc,
    /// Processing an ANSI escape sequence
    Ansi,
    /// Processing an ANSI control string
    AnsiString,
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
}

impl Phase {
    pub const fn is_mxp(self) -> bool {
        matches!(
            self,
            Self::MxpElement | Self::MxpComment | Self::MxpQuote | Self::MxpEntity
        )
    }

    pub const fn is_phase_reset(self, c: u8) -> bool {
        match self {
            Self::Sb | Self::Subnegotiation | Self::SubnegotiationIac | Self::AnsiString => false,
            Self::Iac => matches!(c, b'\r' | b'\n' | ansi::ESC),
            _ => matches!(c, b'\r' | b'\n' | ansi::ESC | telnet::IAC),
        }
    }
}
