use std::fmt;

use mxp::escape::ansi::CSI;

/// Formats a DECSRC response.
pub(crate) struct SecureResetConfirmation {
    pub sequence: u16,
}

impl fmt::Display for SecureResetConfirmation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { sequence } = self;
        write!(f, "{CSI}{sequence}*q")
    }
}
