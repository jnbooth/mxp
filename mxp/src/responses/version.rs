use std::fmt;

use crate::VERSION;

/// Formats a [`<VERSION>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control) response.
pub struct VersionResponse<'a> {
    pub name: &'a str,
    pub version: &'a str,
}

impl fmt::Display for VersionResponse<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\x1B[1z<VERSION MXP=\"{VERSION}\" CLIENT={} VERSION=\"{}\" REGISTERED=yes>",
            self.name, self.version
        )
    }
}
