use std::str;

use casefold::ascii::CaseFold;
use flagset::FlagSet;

use crate::lookup::Lookup;

use super::action::ActionKind;

/// Atomic MXP tags that we recognise, e.g. <b>.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Atom {
    /// Tag name, e.g. bold
    pub name: String,
    /// Whether the atom is open (OPEN)
    pub open: bool,
    /// Whether the atom has no closing tag (EMPTY)
    pub command: bool,
    /// Its action.
    pub action: ActionKind,
    /// Supported arguments, e.g. href, hint
    pub args: Vec<&'static CaseFold<str>>,
}

impl Atom {
    pub fn get(name: &str) -> Option<&'static Self> {
        ALL_ATOMS.get(name)
    }

    pub fn fmt_supported<I>(buf: &mut Vec<u8>, iter: I, supported: FlagSet<ActionKind>)
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        buf.extend_from_slice(b"\x1B[1z<SUPPORTS ");
        let mut has_args = false;
        for arg in iter {
            has_args = true;
            write_supported(buf, supported, arg.as_ref());
        }
        write_supported_suffix(buf, supported, has_args);
    }
}

fn write_supported(buf: &mut Vec<u8>, supported: FlagSet<ActionKind>, arg: &str) {
    let mut questions = arg.split('.');
    let tag = questions.next().unwrap();
    match Atom::get(tag) {
        None => write_cant(buf, tag),
        Some(atom) if !supported.contains(atom.action) => {
            write_cant(buf, tag);
        }
        Some(atom) => match questions.next() {
            None => write_can(buf, tag),
            Some("*") => write_can_args(buf, atom),
            Some(subtag) if atom.args.contains(&subtag.into()) => {
                write_can(buf, subtag);
            }
            Some(subtag) => write_cant(buf, subtag),
        },
    }
}

fn write_supported_suffix(buf: &mut Vec<u8>, supported: FlagSet<ActionKind>, has_args: bool) {
    if !has_args {
        for atom in ALL_ATOMS.values() {
            if supported.contains(atom.action) {
                write_can(buf, &atom.name);
                write_can_args(buf, atom);
            }
        }
    }
    if !supported.contains(ActionKind::Font) && supported.contains(ActionKind::Color) {
        let simple_font = Atom {
            args: vec!["color".into(), "back".into()],
            ..ALL_ATOMS.get("font").unwrap().clone()
        };
        write_can(buf, &simple_font.name);
        write_can_args(buf, &simple_font);
    }
    buf.extend_from_slice(b">\n");
}

fn write_cant(buf: &mut Vec<u8>, tag: &str) {
    buf.push(b'-');
    buf.extend_from_slice(tag.as_bytes());
    buf.push(b' ');
}

fn write_can(buf: &mut Vec<u8>, tag: &str) {
    buf.push(b'+');
    buf.extend_from_slice(tag.as_bytes());
    buf.push(b' ');
}

fn write_can_args(buf: &mut Vec<u8>, atom: &Atom) {
    let name = atom.name.as_bytes();
    for arg in &atom.args {
        buf.push(b'+');
        buf.extend_from_slice(name);
        buf.push(b'.');
        buf.extend_from_slice(arg.as_str().as_bytes());
        buf.push(b' ');
    }
}

#[allow(clippy::enum_glob_use)]
static ALL_ATOMS: Lookup<Atom> = Lookup::new(|| {
    use ActionKind::*;

    let command: FlagSet<ActionKind> = Br
        | Expire
        | Filter
        | Gauge
        | Hr
        | Music
        | Mxp
        | NoBr
        | Password
        | Relocate
        | Reset
        | SBr
        | Stat
        | Support
        | User
        | Version
        | Frame
        | Image
        | Music
        | Sound;

    let open: FlagSet<ActionKind> =
        Bold | Color | Italic | Highlight | Strikeout | Small | Tt | Underline | Font;

    [
        ("a", Hyperlink, "href hint expire"),
        ("b", Bold, ""),
        ("bold", Bold, ""),
        ("br", Br, ""),
        ("c", Color, "fore back"),
        ("color", Color, "fore back"),
        ("dest", Dest, ""),
        ("destination", Dest, ""),
        ("em", Italic, ""),
        ("expire", Expire, ""),
        ("filter", Filter, ""),
        ("font", Font, "face size color back"),
        (
            "frame",
            Frame,
            "name action title internal align left top width height scrolling floating",
        ),
        ("gauge", Gauge, ""),
        ("h", Highlight, ""),
        ("h1", H1, ""),
        ("h2", H2, ""),
        ("h3", H3, ""),
        ("h4", H4, ""),
        ("h5", H5, ""),
        ("h6", H6, ""),
        ("high", Highlight, ""),
        ("hr", Hr, ""),
        ("i", Italic, ""),
        ("image", Image, "url fname t h w hspace vspace align ismap"),
        ("italic", Italic, ""),
        ("music", Music, "fname v l c t u"),
        ("music", Sound, ""),
        ("mxp", Mxp, "off"),
        ("nobr", NoBr, ""),
        ("p", P, ""),
        ("pass", Password, ""),
        ("password", Password, ""),
        ("relocate", Relocate, ""),
        ("reset", Reset, ""),
        ("s", Strikeout, ""),
        ("sbr", SBr, ""),
        ("send", Send, "href hint prompt expire"),
        ("small", Small, ""),
        ("sound", Sound, "fname v l p t u"),
        ("stat", Stat, ""),
        ("strike", Strikeout, ""),
        ("strikeout", Strikeout, ""),
        ("strong", Bold, ""),
        ("support", Support, ""),
        ("tt", Tt, ""),
        ("u", Underline, ""),
        ("underline", Underline, ""),
        ("user", User, ""),
        ("username", User, ""),
        ("v", Var, ""),
        ("var", Var, ""),
        ("version", Version, ""),
    ]
    .into_iter()
    .map(|(name, action, args)| {
        let args = if args.is_empty() {
            Vec::new()
        } else {
            args.split(' ').map(CaseFold::borrow).collect()
        };
        let atom = Atom {
            name: name.to_owned(),
            command: command.contains(action),
            open: open.contains(action),
            action,
            args,
        };
        (name, atom)
    })
    .collect()
});
