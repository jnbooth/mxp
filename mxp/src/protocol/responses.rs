use crate::VERSION;
use std::fmt::{self, Display, Formatter};

pub struct IdentifyResponse<'a> {
    pub name: &'a str,
    pub version: &'a str,
}

impl<'a> Display for IdentifyResponse<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\x1B[1z<VERSION MXP=\"{VERSION}\" CLIENT={} VERSION=\"{}\" REGISTERED=yes>",
            self.name, self.version
        )
    }
}
