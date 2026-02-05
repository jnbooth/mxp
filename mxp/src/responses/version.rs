use std::fmt;

use crate::VERSION;

/// Formats a [`<VERSION>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control) response.
#[derive(Copy, Clone, Debug)]
pub struct VersionResponse<'a> {
    pub name: &'a str,
    pub version: &'a str,
}

impl fmt::Display for VersionResponse<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { name, version } = self;
        write!(
            f,
            "\x1B[1z<VERSION MXP=\"{VERSION}\" CLIENT={name} VERSION=\"{version}\" REGISTERED=yes>",
        )
    }
}
