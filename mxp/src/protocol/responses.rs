use std::fmt;

use casefold::ascii::CaseFold;
use flagset::FlagSet;

use crate::VERSION;
use crate::element::{ActionKind, Tag};

/// Formats a [`<SUPPORT>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control) response.
pub struct SupportResponse<I>
where
    I: IntoIterator + Copy,
    I::Item: AsRef<str>,
{
    iter: I,
    supported: FlagSet<ActionKind>,
}

impl<I> SupportResponse<I>
where
    I: IntoIterator + Copy,
    I::Item: AsRef<str>,
{
    pub fn new(iter: I, supported: FlagSet<ActionKind>) -> Self {
        Self { iter, supported }
    }

    fn write_supported(&self, f: &mut fmt::Formatter, arg: &str) -> fmt::Result {
        let mut questions = arg.split('.');
        let tag_name = questions.next().unwrap();
        match Tag::well_known(tag_name) {
            None => Self::write_cant(f, tag_name),
            Some(tag) if !self.supported.contains(tag.action) => Self::write_cant(f, tag_name),
            Some(tag) => match questions.next() {
                None => Self::write_can(f, tag_name),
                Some("*") => Self::write_can_args(f, tag),
                Some(subtag) if tag.args.contains(&subtag.into()) => Self::write_can(f, subtag),
                Some(subtag) => Self::write_cant(f, subtag),
            },
        }
    }

    fn write_supported_suffix(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /// Alternative `<font>` definition that does not include the "face" and "size" arguments.
        const SIMPLE_FONT_TAG: &Tag = &Tag::new(
            "font",
            ActionKind::Font,
            &[CaseFold::borrow("color"), CaseFold::borrow("back")],
        );

        for tag in Tag::supported() {
            if self.supported.contains(tag.action) {
                Self::write_can(f, tag.name)?;
                Self::write_can_args(f, tag)?;
            }
        }
        if !self.supported.contains(ActionKind::Font) && self.supported.contains(ActionKind::Color)
        {
            Self::write_can(f, SIMPLE_FONT_TAG.name)?;
            Self::write_can_args(f, SIMPLE_FONT_TAG)?;
        }
        Ok(())
    }

    fn write_cant(f: &mut fmt::Formatter, tag: &str) -> fmt::Result {
        write!(f, "-{tag} ")
    }

    fn write_can(f: &mut fmt::Formatter, tag: &str) -> fmt::Result {
        write!(f, "+{tag} ")
    }

    fn write_can_args(f: &mut fmt::Formatter, tag: &Tag) -> fmt::Result {
        let name = tag.name;
        for arg in tag.args {
            write!(f, "+{name}.{arg} ")?;
        }
        Ok(())
    }
}

impl<I> fmt::Display for SupportResponse<I>
where
    I: IntoIterator + Copy,
    I::Item: AsRef<str>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1B[1z<SUPPORTS ")?;
        let mut has_args = false;
        for arg in self.iter {
            has_args = true;
            self.write_supported(f, arg.as_ref())?;
        }
        if !has_args {
            self.write_supported_suffix(f)?;
        }
        write!(f, ">")
    }
}

/// Formats a [`<VERSION>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control) response.
pub struct VersionResponse<'a> {
    pub name: &'a str,
    pub version: &'a str,
}

impl fmt::Display for VersionResponse<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\x1B[1z<VERSION MXP=\"{VERSION}\" CLIENT={} VERSION=\"{}\" REGISTERED=yes>",
            self.name, self.version
        )
    }
}
