use std::str;

use casefold::ascii::{CaseFold, CaseFoldMap};
use flagset::FlagSet;

use super::action::ActionKind;

/// Atomic MXP tags that we recognise, e.g. <b>.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag {
    /// Tag name, e.g. bold
    pub name: &'static str,
    /// Its action.
    pub action: ActionKind,
    /// Supported arguments, e.g. href, hint
    pub args: &'static [&'static CaseFold<str>],
}

impl Tag {
    const fn new(
        name: &'static str,
        action: ActionKind,
        args: &'static [&'static CaseFold<str>],
    ) -> Self {
        Self { name, action, args }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Tags {
    inner: CaseFoldMap<&'static str, &'static Tag>,
}

impl Tags {
    pub fn well_known() -> Self {
        Self {
            inner: ALL_TAGS.iter().map(|tag| (tag.name.into(), tag)).collect(),
        }
    }

    pub fn get(&self, tag: &str) -> Option<&'static Tag> {
        self.inner.get(tag).copied()
    }

    pub fn fmt_supported<I>(&self, buf: &mut Vec<u8>, iter: I, supported: FlagSet<ActionKind>)
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        buf.extend_from_slice(b"\x1B[1z<SUPPORTS ");
        let mut has_args = false;
        for arg in iter {
            has_args = true;
            self.write_supported(buf, supported, arg.as_ref());
        }
        self.write_supported_suffix(buf, supported, has_args);
    }

    fn write_supported(&self, buf: &mut Vec<u8>, supported: FlagSet<ActionKind>, arg: &str) {
        let mut questions = arg.split('.');
        let tag_name = questions.next().unwrap();
        match self.get(tag_name) {
            None => Self::write_cant(buf, tag_name),
            Some(tag) if !supported.contains(tag.action) => {
                Self::write_cant(buf, tag_name);
            }
            Some(tag) => match questions.next() {
                None => Self::write_can(buf, tag_name),
                Some("*") => Self::write_can_args(buf, tag),
                Some(subtag) if tag.args.contains(&subtag.into()) => {
                    Self::write_can(buf, subtag);
                }
                Some(subtag) => Self::write_cant(buf, subtag),
            },
        }
    }

    fn write_supported_suffix(
        &self,
        buf: &mut Vec<u8>,
        supported: FlagSet<ActionKind>,
        has_args: bool,
    ) {
        if !has_args {
            for tag in self.inner.values() {
                if supported.contains(tag.action) {
                    Self::write_can(buf, tag.name);
                    Self::write_can_args(buf, tag);
                }
            }
        }
        if !supported.contains(ActionKind::Font) && supported.contains(ActionKind::Color) {
            Self::write_can(buf, SIMPLE_FONT_TAG.name);
            Self::write_can_args(buf, SIMPLE_FONT_TAG);
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

    fn write_can_args(buf: &mut Vec<u8>, tag: &Tag) {
        let name = tag.name.as_bytes();
        for arg in tag.args {
            buf.push(b'+');
            buf.extend_from_slice(name);
            buf.push(b'.');
            buf.extend_from_slice(arg.as_str().as_bytes());
            buf.push(b' ');
        }
    }
}

macro_rules! args {
        ($($l:literal),+ $(,)?) => (&[$(CaseFold::borrow($l)),+])
    }

const ALL_TAGS: &[Tag] = {
    #[allow(clippy::enum_glob_use)]
    use ActionKind::*;

    &[
        Tag::new("a", Hyperlink, args!["href", "hint", "expire"]),
        Tag::new("b", Bold, &[]),
        Tag::new("bold", Bold, &[]),
        Tag::new("br", Br, &[]),
        Tag::new("c", Color, args!["fore", "back"]),
        Tag::new("color", Color, args!["fore", "back"]),
        Tag::new("dest", Dest, &[]),
        Tag::new("destination", Dest, &[]),
        Tag::new("em", Italic, &[]),
        Tag::new("expire", Expire, &[]),
        Tag::new("filter", Filter, &[]),
        Tag::new("font", Font, args!["face", "size", "color", "back"]),
        Tag::new(
            "frame",
            Frame,
            args![
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
        Tag::new("gauge", Gauge, &[]),
        Tag::new("h", Highlight, &[]),
        Tag::new("h1", H1, &[]),
        Tag::new("h2", H2, &[]),
        Tag::new("h3", H3, &[]),
        Tag::new("h4", H4, &[]),
        Tag::new("h5", H5, &[]),
        Tag::new("h6", H6, &[]),
        Tag::new("high", Highlight, &[]),
        Tag::new("hr", Hr, &[]),
        Tag::new("i", Italic, &[]),
        Tag::new(
            "image",
            Image,
            args!["url", "fname", "t", "h", "w", "hspace", "vspace", "align", "ismap"],
        ),
        Tag::new("italic", Italic, &[]),
        Tag::new("music", Music, args!["fname", "v", "l", "c", "t", "u"]),
        Tag::new("music", Sound, &[]),
        Tag::new("mxp", Mxp, args!["off"]),
        Tag::new("nobr", NoBr, &[]),
        Tag::new("p", P, &[]),
        Tag::new("pass", Password, &[]),
        Tag::new("password", Password, &[]),
        Tag::new("relocate", Relocate, &[]),
        Tag::new("reset", Reset, &[]),
        Tag::new("s", Strikeout, &[]),
        Tag::new("sbr", SBr, &[]),
        Tag::new("send", Send, args!["href", "hint", "prompt", "expire"]),
        Tag::new("small", Small, &[]),
        Tag::new("sound", Sound, args!["fname", "v", "l", "p", "t", "u"]),
        Tag::new("stat", Stat, &[]),
        Tag::new("strike", Strikeout, &[]),
        Tag::new("strikeout", Strikeout, &[]),
        Tag::new("strong", Bold, &[]),
        Tag::new("support", Support, &[]),
        Tag::new("tt", Tt, &[]),
        Tag::new("u", Underline, &[]),
        Tag::new("underline", Underline, &[]),
        Tag::new("user", User, &[]),
        Tag::new("username", User, &[]),
        Tag::new("v", Var, &[]),
        Tag::new("var", Var, &[]),
        Tag::new("version", Version, &[]),
    ]
};

/// Alternative `<font>` definition that does not include the "face" and "size" arguments.
const SIMPLE_FONT_TAG: &Tag = &Tag::new("font", ActionKind::Font, args!["color", "back"]);
