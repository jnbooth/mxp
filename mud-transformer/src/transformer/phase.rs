use mxp::escape::telnet;

const fn is_phase_reset_character(c: u8) -> bool {
    matches!(c, b'\r' | b'\n' | telnet::ESC | telnet::IAC)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(crate) enum Phase {
    /// Normal text
    #[default]
    Normal,
    /// Received an escape
    Esc,
    /// Processing an ANSI escape sequence
    Ansi,
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
