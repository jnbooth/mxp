pub fn identify<'a>(name: &'a str, version: &'a str) -> [&'a str; 7] {
    [
        "\x1B[1z<VERSION MXP=\"",
        crate::VERSION,
        "\" CLIENT=",
        name,
        " VERSION=\"",
        version,
        "\" REGISTERED=YES>\n",
    ]
}

pub fn afk<'a>(afk_seconds: &'a str, challenge: &'a str) -> [&'a str; 4] {
    ["\x1B[1z<AFK ", afk_seconds, challenge, ">\n"]
}
