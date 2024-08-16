use std::str;

use casefold::ascii::CaseFold;
use enumeration::{self, enums, Enum, EnumSet};

use crate::lookup::Lookup;

use super::action::ActionKind;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum TagFlag {
    /// Tag is an open one (otherwise secure)
    Open,
    /// Tag is a command (doesn't have closing tag)
    Command,
    /// Not closed by reset (eg. body)
    NoReset,
    /// Not really implemented (for <supports> tag)
    NotImp,
}

/// Atomic MXP tags that we recognise, e.g. <b>.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Atom {
    /// Tag name, e.g. bold
    pub name: String,
    /// Secure, Command, etc.
    pub flags: EnumSet<TagFlag>,
    /// Its action.
    pub action: ActionKind,
    /// Supported arguments, e.g. href, hint
    pub args: Vec<&'static CaseFold<str>>,
}

impl Atom {
    pub fn get(name: &str) -> Option<&'static Self> {
        ALL_ATOMS.get(name)
    }

    pub fn fmt_supported<I>(buf: &mut Vec<u8>, iter: I, unsupported: EnumSet<ActionKind>)
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        buf.extend_from_slice(b"\x1B[1z<SUPPORTS ");
        let mut has_args = false;
        for arg in iter {
            has_args = true;
            let mut questions = arg.as_ref().split('.');
            let tag = questions.next().unwrap();
            match Atom::get(tag) {
                None => write_cant(buf, tag),
                Some(atom)
                    if atom.flags.contains(TagFlag::NotImp) | unsupported.contains(atom.action) =>
                {
                    write_cant(buf, tag);
                }
                Some(atom) => match questions.next() {
                    None => write_can(buf, tag),
                    Some("*") => write_can_args(buf, atom),
                    Some(subtag) if atom.args.contains(&CaseFold::borrow(subtag)) => {
                        write_can(buf, subtag);
                    }
                    Some(subtag) => write_cant(buf, subtag),
                },
            }
        }
        if !has_args {
            for atom in ALL_ATOMS.values() {
                if atom.flags.contains(TagFlag::NotImp) | unsupported.contains(atom.action) {
                    write_cant(buf, &atom.name);
                } else {
                    write_can(buf, &atom.name);
                    write_can_args(buf, atom);
                }
            }
        }
        buf.extend_from_slice(b">\n");
    }
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
    use TagFlag::*;

    let atom = |name: &'static str, flags, action, args: &[&'static str]| {
        let atom = Atom {
            name: name.to_owned(),
            flags,
            action,
            args: args.iter().map(|&s| CaseFold::borrow(s)).collect(),
        };
        (name, atom)
    };

    vec![
        atom("a", enums![], Hyperlink, &["href", "hint", "expire"]),
        atom("afk", enums![Command], Afk, &["challenge"]),
        atom("b", enums![Open], Bold, &[]),
        atom("bold", enums![Open], Bold, &[]),
        atom("br", enums![Command], Br, &[]),
        atom("c", enums![Open], Color, &["fore", "back"]),
        atom("center", enums![NotImp], Center, &[]),
        atom("color", enums![Open], Color, &["fore", "back"]),
        atom("dest", enums![], Dest, &[]),
        atom("em", enums![Open], Italic, &[]),
        atom("expire", enums![], Expire, &[]),
        atom("filter", enums![NotImp], Filter, &[]),
        atom("gauge", enums![NotImp], Gauge, &[]),
        atom("h", enums![Open], High, &[]),
        atom("h1", enums![], H1, &[]),
        atom("h2", enums![], H2, &[]),
        atom("h3", enums![], H3, &[]),
        atom("h4", enums![], H4, &[]),
        atom("h5", enums![], H5, &[]),
        atom("h6", enums![], H6, &[]),
        atom("high", enums![Open], High, &[]),
        atom("hr", enums![Command], Hr, &[]),
        atom("i", enums![Open], Italic, &[]),
        atom("italic", enums![Open], Italic, &[]),
        atom("li", enums![Command], Li, &[]),
        atom("music", enums![Command], Sound, &[]),
        atom("mxp", enums![Command], Mxp, &["off"]),
        atom("nobr", enums![], NoBr, &[]),
        atom("ol", enums![], Ol, &[]),
        atom("option", enums![Command], SetOption, &[]),
        atom("p", enums![], P, &[]),
        atom("pass", enums![Command], Password, &[]),
        atom("password", enums![Command], Password, &[]),
        atom("recommend_option", enums![Command], RecommendOption, &[]),
        atom("relocate", enums![Command, NotImp], Relocate, &[]),
        atom("reset", enums![Command], Reset, &[]),
        atom("s", enums![Open, NotImp], Strikeout, &[]),
        atom("sbr", enums![Open], SBr, &[]),
        atom("samp", enums![], Samp, &[]),
        atom("script", enums![NotImp], Script, &[]),
        atom("small", enums![Open, NotImp], Small, &[]),
        atom("stat", enums![NotImp], Stat, &[]),
        atom("strike", enums![Open, NotImp], Strikeout, &[]),
        atom("strikeout", enums![Open, NotImp], Strikeout, &[]),
        atom("strong", enums![Open], Bold, &[]),
        atom("support", enums![Command], Support, &[]),
        atom("tt", enums![Open, NotImp], Tt, &[]),
        atom("u", enums![Open], Underline, &[]),
        atom("ul", enums![], Ul, &[]),
        atom("underline", enums![Open], Underline, &[]),
        atom("user", enums![Command], User, &[]),
        atom("username", enums![Command], User, &[]),
        atom("v", enums![], Var, &[]),
        atom("var", enums![], Var, &[]),
        atom("version", enums![Command], Version, &[]),
        atom(
            "font",
            enums![Open],
            Font,
            &["face", "size", "color", "back"],
        ),
        atom(
            "frame",
            enums![],
            Frame,
            &[
                "name",
                "action",
                "title",
                "internal",
                "align",
                "left",
                "top",
                "width",
                "height",
                "scrolling",
                "floating",
            ],
        ),
        atom(
            "image",
            enums![Command, NotImp],
            Image,
            &[
                "url", "fname", "t", "h", "w", "hspace", "vspace", "align", "ismap",
            ],
        ),
        atom(
            "music",
            enums![Command],
            Music,
            &["fname", "v", "l", "c", "t", "u"],
        ),
        atom(
            "send",
            enums![],
            Send,
            &["href", "hint", "prompt", "expire"],
        ),
        atom(
            "sound",
            enums![Command],
            Sound,
            &["fname", "v", "l", "p", "t", "u"],
        ),
    ]
});
