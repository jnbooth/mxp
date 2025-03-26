use std::fmt::{self, Display, Formatter};
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

    pub fn supported<I>(&self, iter: I, supported: FlagSet<ActionKind>) -> SupportedTags<I>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        SupportedTags {
            supported,
            iter,
            tags: self,
        }
    }

    fn write_supported(
        &self,
        f: &mut Formatter,
        supported: FlagSet<ActionKind>,
        arg: &str,
    ) -> fmt::Result {
        let mut questions = arg.split('.');
        let tag_name = questions.next().unwrap();
        match self.get(tag_name) {
            None => Self::write_cant(f, tag_name),
            Some(tag) if !supported.contains(tag.action) => Self::write_cant(f, tag_name),
            Some(tag) => match questions.next() {
                None => Self::write_can(f, tag_name),
                Some("*") => Self::write_can_args(f, tag),
                Some(subtag) if tag.args.contains(&subtag.into()) => Self::write_can(f, subtag),
                Some(subtag) => Self::write_cant(f, subtag),
            },
        }
    }

    fn write_supported_suffix(
        &self,
        f: &mut Formatter,
        supported: FlagSet<ActionKind>,
    ) -> fmt::Result {
        for tag in self.inner.values() {
            if supported.contains(tag.action) {
                Self::write_can(f, tag.name)?;
                Self::write_can_args(f, tag)?;
            }
        }
        if !supported.contains(ActionKind::Font) && supported.contains(ActionKind::Color) {
            Self::write_can(f, SIMPLE_FONT_TAG.name)?;
            Self::write_can_args(f, SIMPLE_FONT_TAG)?;
        }
        Ok(())
    }

    fn write_cant(f: &mut Formatter, tag: &str) -> fmt::Result {
        write!(f, "-{tag} ")
    }

    fn write_can(f: &mut Formatter, tag: &str) -> fmt::Result {
        write!(f, "+{tag} ")
    }

    fn write_can_args(f: &mut Formatter, tag: &Tag) -> fmt::Result {
        let name = tag.name;
        for arg in tag.args {
            write!(f, "+{name}.{arg} ")?;
        }
        Ok(())
    }
}

pub struct SupportedTags<'a, I> {
    supported: FlagSet<ActionKind>,
    iter: I,
    tags: &'a Tags,
}

impl<'a, I> Display for SupportedTags<'a, I>
where
    I: IntoIterator + Copy,
    I::Item: AsRef<str>,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "\x1B[1z<SUPPORTS ")?;
        let mut has_args = false;
        for arg in self.iter {
            has_args = true;
            self.tags.write_supported(f, self.supported, arg.as_ref())?;
        }
        if !has_args {
            self.tags.write_supported_suffix(f, self.supported)?;
        }
        write!(f, ">")
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
