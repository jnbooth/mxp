use std::str;
use std::sync::OnceLock;

use casefold::ascii::{CaseFold, CaseFoldMap};
use enumeration::{self, enums, Enum, EnumSet};

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
    fn all_atoms() -> &'static AtomMap {
        static ALL_ATOMS: OnceLock<AtomMap> = OnceLock::new();
        ALL_ATOMS.get_or_init(create_atoms)
    }

    pub fn get(name: &str) -> Option<&'static Self> {
        Self::all_atoms().get(name)
    }

    pub fn fmt_supported(buf: &mut Vec<u8>, args: Arguments) {
        buf.extend_from_slice(b"\x1B[1z<SUPPORTS ");
        if args.is_empty() {
            for atom in Self::all_atoms().values() {
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

type AtomMap = CaseFoldMap<String, Atom>;

fn create_atoms() -> AtomMap {
    let mut all: AtomMap = CaseFoldMap::new();
    let mut add = |name: &'static str, flags, action, args: &[&'static str]| {
        all.insert(
            name.to_owned(),
            Atom {
                name: name.to_owned(),
                flags,
                action,
                args: args.iter().map(|&s| CaseFold::borrow(s)).collect(),
            },
        )
    };

    use Action::*;
    use TagFlag::*;
    add("a", enums![], Hyperlink, &["href", "xch_cmd", "xch_hint"]);
    add("afk", enums![Command], Afk, &[]);
    add("b", enums![Open], Bold, &[]);
    add("body", enums![Pueblo, NoReset], Body, &[]);
    add("bold", enums![Open], Bold, &[]);
    add("br", enums![Command], Br, &[]);
    add("c", enums![Open], Color, &["fore", "back"]);
    add("center", enums![NotImp], Center, &[]);
    add("color", enums![Open], Color, &["fore", "back"]);
    add("dest", enums![NotImp], Dest, &[]);
    add("em", enums![Open], Italic, &[]);
    add("expire", enums![NotImp], Expire, &[]);
    add("filter", enums![NotImp], Filter, &[]);
    add("frame", enums![NotImp], Frame, &[]);
    add("gauge", enums![NotImp], Gauge, &[]);
    add("h", enums![Open], High, &[]);
    add("h1", enums![NotImp], H1, &[]);
    add("h2", enums![NotImp], H2, &[]);
    add("h3", enums![NotImp], H3, &[]);
    add("h4", enums![NotImp], H4, &[]);
    add("h5", enums![NotImp], H5, &[]);
    add("h6", enums![NotImp], H6, &[]);
    add("head", enums![Pueblo, NoReset], Head, &[]);
    add("high", enums![Open], High, &[]);
    add("hr", enums![Command], Hr, &[]);
    add("html", enums![Pueblo, NoReset], Html, &[]);
    add("i", enums![Open], Italic, &[]);
    add("image", enums![Command, NotImp], Image, &["url", "fname"]);
    add("img", enums![Pueblo, Command], Img, &["src", "xch_mode"]);
    add("italic", enums![Open], Italic, &[]);
    add("li", enums![Command], Li, &[]);
    add("music", enums![Command, NotImp], Sound, &[]);
    add("mxp", enums![Command], Mxp, &["off"]);
    add("nobr", enums![NotImp], NoBr, &[]);
    add("ol", enums![], Ol, &[]);
    add("option", enums![Command], SetOption, &[]);
    add("p", enums![], P, &[]);
    add("pass", enums![Command], Password, &[]);
    add("password", enums![Command], Password, &[]);
    add("pre", enums![Pueblo], Pre, &[]);
    add("recommend_option", enums![Command], RecommendOption, &[]);
    add("relocate", enums![Command, NotImp], Relocate, &[]);
    add("reset", enums![Command], Reset, &[]);
    add("s", enums![Open, NotImp], Strike, &[]);
    add("samp", enums![], Samp, &[]);
    add("script", enums![NotImp], Script, &[]);
    add("small", enums![Open, NotImp], Small, &[]);
    add("sound", enums![Command, NotImp], Sound, &[]);
    add("stat", enums![NotImp], Stat, &[]);
    add("strike", enums![Open, NotImp], Strike, &[]);
    add("strong", enums![Open], Bold, &[]);
    add("support", enums![Command], Support, &[]);
    add("title", enums![Pueblo], Title, &[]);
    add("tt", enums![Open, NotImp], Tt, &[]);
    add("u", enums![Open], Underline, &[]);
    add("ul", enums![], Ul, &[]);
    add("underline", enums![Open], Underline, &[]);
    add("user", enums![Command], User, &[]);
    add("username", enums![Command], User, &[]);
    add("v", enums![], Var, &[]);
    add("var", enums![], Var, &[]);
    add("version", enums![Command], Version, &[]);
    add("xch_page", enums![Pueblo, Command], XchPage, &[]);
    add("xch_pane", enums![Pueblo, Command, NotImp], XchPane, &[]);
    add(
        "font",
        enums![Open],
        Font,
        &["color", "back", "fgcolor", "bgcolor"],
    );
    add(
        "send",
        enums![],
        Send,
        &["href", "hint", "xch_cmd", "xch_hint", "prompt"],
    );

    all
}
