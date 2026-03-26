/// MUD Sound Protocol
///
/// https://www.zuggsoft.com/zmud/msp.htm
pub const OPT: u8 = 90;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Element<S> {
    Music(mxp::Music<S>),
    Sound(mxp::Sound<S>),
}

pub fn parse(s: &str) -> mxp::Result<Element<&str>> {
    let source = s.strip_prefix("!!").unwrap_or(s);
    let source = source.strip_suffix(')').ok_or_else(|| {
        mxp::Error::new(
            format!("no closing parenthesis at end of '{s}'"),
            mxp::ErrorKind::IncompleteElement,
        )
    })?;
    match source.split_once('(') {
        Some(("MUSIC", args)) => mxp::Music::from_msp(args).map(Element::Music),
        Some(("SOUND", args)) => mxp::Sound::from_msp(args).map(Element::Sound),
        Some((name, _)) => Err(mxp::Error::new(
            format!("expected MUSIC or SOUND, got {name}"),
            mxp::ErrorKind::UnknownElement,
        )),
        None => Err(mxp::Error::new(
            format!("no opening parenthesis for '{s}'"),
            mxp::ErrorKind::IncompleteElement,
        )),
    }
}
