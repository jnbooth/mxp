use std::str;

use casefold::ascii::CaseFold;
use enumeration::{self, enums, Enum, EnumSet};

use crate::lookup::Lookup;

use super::argument::Arguments;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum TagFlag {
    /// Tag is an open one (otherwise secure)
    Open,
    /// Tag is a command (doesn't have closing tag)
    Command,
    /// Tag is Pueblo-only
    Pueblo,
    /// Not closed by reset (eg. body)
    NoReset,
    /// Not really implemented (for <supports> tag)
    NotImp,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Action {
    /// eg. <send href="go west"> west
    Send,
    /// bold
    Bold,
    /// underline
    Underline,
    /// italic
    Italic,
    /// eg. <color fore=red back=blue>
    Color,
    /// version request
    Version,
    /// Font appearance
    Font,
    /// play sound
    Sound,
    /// send username
    User,
    /// send password
    Password,
    /// causes a new connect to open
    Relocate,
    /// frame
    Frame,
    /// destination frame
    Dest,
    /// show image
    Image,
    /// sound/image filter
    Filter,
    /// Hyperlink (secure)
    Hyperlink,
    /// Hard Line break (secure)
    Br,
    /// Level 1 heading (secure)
    H1,
    /// Level 2 heading (secure)
    H2,
    /// Level 3 heading (secure)
    H3,
    /// Level 4 heading (secure)
    H4,
    /// Level 5 heading (secure)
    H5,
    /// Level 6 heading (secure)
    H6,
    /// Horizontal rule (secure)
    Hr,
    /// non-breaking newline
    NoBr,
    /// Paragraph break (secure)
    P,
    /// Strikethrough
    Strike,
    /// Client script (secure)
    Script,
    /// Small text
    Small,
    /// Non-proportional font
    Tt,
    /// Unordered list
    Ul,
    /// Ordered list
    Ol,
    /// List item
    Li,
    /// Sample text
    Samp,
    /// Centre text
    Center,
    /// Highlight text
    High,
    /// Set variable
    Var,
    /// AFK - away from keyboard time
    Afk,

    // recent
    /// gauge
    Gauge,
    /// status
    Stat,
    /// expire
    Expire,

    /// close all open tags
    Reset,
    /// MXP command (eg. MXP OFF)
    Mxp,
    /// what commands we support
    Support,

    /// client options set
    SetOption,
    /// server sets option
    RecommendOption,

    // Pueblo
    /// Preformatted text
    Pre,
    Body,
    Head,
    Html,
    Title,
    Img,
    XchPage,
    XchPane,
}

/// Atomic MXP tags that we recognise, e.g. <b>.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Atom {
    /// Tag name, e.g. bold
    pub name: String,
    /// Secure, Command, etc.
    pub flags: EnumSet<TagFlag>,
    /// Its action.
    pub action: Action,
    /// Supported arguments, e.g. href, hint
    pub args: Vec<&'static CaseFold<str>>,
}

impl Atom {
    pub fn get(name: &str) -> Option<&'static Self> {
        ALL_ATOMS.get(name)
    }

    pub fn fmt_supported(buf: &mut Vec<u8>, args: Arguments) {
        buf.extend_from_slice(b"\x1B[1z<SUPPORTS ");
        if args.is_empty() {
            for atom in ALL_ATOMS.values() {
                write_can(buf, &atom.name);
                write_can_args(buf, atom);
            }
        } else {
            for arg in args.values() {
                let mut questions = arg.split('.');
                let tag = questions.next().unwrap();
                match Atom::get(tag) {
                    None => write_cant(buf, tag),
                    Some(atom) if atom.flags.contains(TagFlag::NotImp) => write_cant(buf, tag),
                    Some(atom) => match questions.next() {
                        None => write_can(buf, tag),
                        Some("*") => write_can_args(buf, atom),
                        Some(subtag) if atom.args.contains(&CaseFold::borrow(subtag)) => {
                            write_can(buf, subtag)
                        }
                        Some(subtag) => write_cant(buf, subtag),
                    },
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

static ALL_ATOMS: Lookup<Atom> = Lookup::new(|| {
    let atom = |name: &'static str, flags, action, args: &[&'static str]| {
        let atom = Atom {
            name: name.to_owned(),
            flags,
            action,
            args: args.iter().map(|&s| CaseFold::borrow(s)).collect(),
        };
        (name, atom)
    };

    use Action::*;
    use TagFlag::*;
    vec![
        atom("a", enums![], Hyperlink, &["href", "xch_cmd", "xch_hint"]),
        atom("afk", enums![Command], Afk, &[]),
        atom("b", enums![Open], Bold, &[]),
        atom("body", enums![Pueblo, NoReset], Body, &[]),
        atom("bold", enums![Open], Bold, &[]),
        atom("br", enums![Command], Br, &[]),
        atom("c", enums![Open], Color, &["fore", "back"]),
        atom("center", enums![NotImp], Center, &[]),
        atom("color", enums![Open], Color, &["fore", "back"]),
        atom("dest", enums![NotImp], Dest, &[]),
        atom("em", enums![Open], Italic, &[]),
        atom("expire", enums![NotImp], Expire, &[]),
        atom("filter", enums![NotImp], Filter, &[]),
        atom("frame", enums![NotImp], Frame, &[]),
        atom("gauge", enums![NotImp], Gauge, &[]),
        atom("h", enums![Open], High, &[]),
        atom("h1", enums![NotImp], H1, &[]),
        atom("h2", enums![NotImp], H2, &[]),
        atom("h3", enums![NotImp], H3, &[]),
        atom("h4", enums![NotImp], H4, &[]),
        atom("h5", enums![NotImp], H5, &[]),
        atom("h6", enums![NotImp], H6, &[]),
        atom("head", enums![Pueblo, NoReset], Head, &[]),
        atom("high", enums![Open], High, &[]),
        atom("hr", enums![Command], Hr, &[]),
        atom("html", enums![Pueblo, NoReset], Html, &[]),
        atom("i", enums![Open], Italic, &[]),
        atom("image", enums![Command, NotImp], Image, &["url", "fname"]),
        atom("img", enums![Pueblo, Command], Img, &["src", "xch_mode"]),
        atom("italic", enums![Open], Italic, &[]),
        atom("li", enums![Command], Li, &[]),
        atom("music", enums![Command, NotImp], Sound, &[]),
        atom("mxp", enums![Command], Mxp, &["off"]),
        atom("nobr", enums![NotImp], NoBr, &[]),
        atom("ol", enums![], Ol, &[]),
        atom("option", enums![Command], SetOption, &[]),
        atom("p", enums![], P, &[]),
        atom("pass", enums![Command], Password, &[]),
        atom("password", enums![Command], Password, &[]),
        atom("pre", enums![Pueblo], Pre, &[]),
        atom("recommend_option", enums![Command], RecommendOption, &[]),
        atom("relocate", enums![Command, NotImp], Relocate, &[]),
        atom("reset", enums![Command], Reset, &[]),
        atom("s", enums![Open, NotImp], Strike, &[]),
        atom("samp", enums![], Samp, &[]),
        atom("script", enums![NotImp], Script, &[]),
        atom("small", enums![Open, NotImp], Small, &[]),
        atom("sound", enums![Command, NotImp], Sound, &[]),
        atom("stat", enums![NotImp], Stat, &[]),
        atom("strike", enums![Open, NotImp], Strike, &[]),
        atom("strong", enums![Open], Bold, &[]),
        atom("support", enums![Command], Support, &[]),
        atom("title", enums![Pueblo], Title, &[]),
        atom("tt", enums![Open, NotImp], Tt, &[]),
        atom("u", enums![Open], Underline, &[]),
        atom("ul", enums![], Ul, &[]),
        atom("underline", enums![Open], Underline, &[]),
        atom("user", enums![Command], User, &[]),
        atom("username", enums![Command], User, &[]),
        atom("v", enums![], Var, &[]),
        atom("var", enums![], Var, &[]),
        atom("version", enums![Command], Version, &[]),
        atom("xch_page", enums![Pueblo, Command], XchPage, &[]),
        atom("xch_pane", enums![Pueblo, Command, NotImp], XchPane, &[]),
        atom(
            "font",
            enums![Open],
            Font,
            &["color", "back", "fgcolor", "bgcolor"],
        ),
        atom(
            "send",
            enums![],
            Send,
            &["href", "hint", "xch_cmd", "xch_hint", "prompt"],
        ),
    ]
});
