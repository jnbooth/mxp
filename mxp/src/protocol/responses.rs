use casefold::ascii::CaseFold;
use flagset::FlagSet;

use crate::element::{ActionKind, Tag, Tags};
use crate::VERSION;
use std::fmt::{self, Display, Formatter};

/// Formats a [`<SUPPORT>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control) response.
pub struct SupportResponse<'a, I>
where
    I: IntoIterator + Copy,
    I::Item: AsRef<str>,
{
    iter: I,
    supported: FlagSet<ActionKind>,
    tags: &'a Tags,
}

impl<'a, I> SupportResponse<'a, I>
where
    I: IntoIterator + Copy,
    I::Item: AsRef<str>,
{
    pub(crate) fn new(iter: I, supported: FlagSet<ActionKind>, tags: &'a Tags) -> Self {
        Self {
            iter,
            supported,
            tags,
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
        match self.tags.get(tag_name) {
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
        /// Alternative `<font>` definition that does not include the "face" and "size" arguments.
        const SIMPLE_FONT_TAG: &Tag = &Tag::new(
            "font",
            ActionKind::Font,
            &[CaseFold::borrow("color"), CaseFold::borrow("back")],
        );

        for tag in self.tags {
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

impl<'a, I> Display for SupportResponse<'a, I>
where
    I: IntoIterator + Copy,
    I::Item: AsRef<str>,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "\x1B[1z<SUPPORTS ")?;
        let mut has_args = false;
        for arg in self.iter {
            has_args = true;
            self.write_supported(f, self.supported, arg.as_ref())?;
        }
        if !has_args {
            self.write_supported_suffix(f, self.supported)?;
        }
        write!(f, ">")
    }
}

/// Formats a [`<VERSION>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control) response.
pub struct VersionResponse<'a> {
    pub name: &'a str,
    pub version: &'a str,
}

impl<'a> Display for VersionResponse<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\x1B[1z<VERSION MXP=\"{VERSION}\" CLIENT={} VERSION=\"{}\" REGISTERED=yes>",
            self.name, self.version
        )
    }
}
