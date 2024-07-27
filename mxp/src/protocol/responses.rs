use crate::VERSION;
use std::time::Duration;

pub fn identify(name: &str, version: &str) -> String {
    format!(
        "\x1B[1z<VERSION MXP=\"{VERSION}\" CLIENT={name} VERSION=\"{version}\" REGISTERED=YES\n"
    )
}

pub fn afk(duration: Duration, challenge: &str) -> String {
    format!("\x1B[1z<AFK {}{}\n", duration.as_secs(), challenge)
}
