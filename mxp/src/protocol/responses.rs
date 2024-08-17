use crate::VERSION;
use std::fmt::{self, Display, Formatter};
use std::time::Duration;

pub struct IdentifyResponse<'a> {
    pub name: &'a str,
    pub version: &'a str,
}

impl<'a> Display for IdentifyResponse<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "\x1B[1z<VERSION MXP=\"{VERSION}\" CLIENT={} VERSION=\"{}\" REGISTERED=YES",
            self.name, self.version
        )
    }
}

pub struct AfkResponse<'a> {
    pub duration: Duration,
    pub challenge: &'a str,
}

impl<'a> Display for AfkResponse<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "\x1B[1z<AFK {}{}",
            self.duration.as_secs(),
            self.challenge
        )
    }
}
