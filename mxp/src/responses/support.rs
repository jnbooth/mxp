use std::fmt;

use flagset::FlagSet;

use crate::element::{ActionKind, AtomicTag};

/// Formats a [`<SUPPORT>`] response.
///
/// [`<SUPPORT>`]: https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control
///
/// # Examples
///
/// ```
/// use mxp::responses::SupportResponse;
///
/// let supported = mxp::ActionKind::Bold
///     | mxp::ActionKind::Color
///     | mxp::ActionKind::Image
///     | mxp::ActionKind::Send;
///
/// let response = SupportResponse::new(&["color.*", "send.expire", "image"], supported);
/// assert_eq!(response.to_string(), "\x1B[1z<SUPPORTS +color.fore +color.back +send.expire +image>\r\n");
/// ```
pub struct SupportResponse<I>
where
    I: IntoIterator + Clone,
    I::Item: AsRef<str>,
{
    iter: I,
    supported: FlagSet<ActionKind>,
}

impl<I> SupportResponse<I>
where
    I: IntoIterator + Clone,
    I::Item: AsRef<str>,
{
    pub fn new(iter: I, supported: FlagSet<ActionKind>) -> Self {
        Self { iter, supported }
    }

    fn write_supported(&self, f: &mut fmt::Formatter, query: &str) -> fmt::Result {
        let (name, arg) = match query.split_once('.') {
            Some((name, arg)) => (name, Some(arg)),
            None => (query, None),
        };
        match AtomicTag::well_known(name) {
            Some(tag) if self.supported.contains(tag.action) => match arg {
                None => Self::write_can(f, name),
                Some("*") => Self::write_can_args(f, tag),
                Some(arg) if tag.supports(arg) => Self::write_can_arg(f, name, arg),
                Some(arg) => Self::write_cant_arg(f, name, arg),
            },
            _ => Self::write_cant(f, name),
        }
    }

    fn write_all_supported(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for tag in AtomicTag::supported() {
            if self.supported.contains(tag.action) {
                Self::write_can(f, tag.name)?;
                Self::write_can_args(f, tag)?;
            }
        }
        if !self.supported.contains(ActionKind::Font) && self.supported.contains(ActionKind::Color)
        {
            // Alternative `<font>` definition with only color arguments.
            write!(f, " +font +font.color +font.back")?;
        }
        Ok(())
    }

    fn write_cant(f: &mut fmt::Formatter, tag: &str) -> fmt::Result {
        write!(f, " -{tag}")
    }

    fn write_can(f: &mut fmt::Formatter, tag: &str) -> fmt::Result {
        write!(f, " +{tag}")
    }

    fn write_can_arg(f: &mut fmt::Formatter, tag: &str, arg: &str) -> fmt::Result {
        write!(f, " +{tag}.{arg}")
    }

    fn write_cant_arg(f: &mut fmt::Formatter, tag: &str, arg: &str) -> fmt::Result {
        write!(f, " -{tag}.{arg}")
    }

    fn write_can_args(f: &mut fmt::Formatter, tag: &AtomicTag) -> fmt::Result {
        let name = tag.name;
        for arg in tag.args {
            write!(f, " +{name}.{arg}")?;
        }
        Ok(())
    }
}

impl<I> fmt::Display for SupportResponse<I>
where
    I: IntoIterator + Clone,
    I::Item: AsRef<str>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1B[1z<SUPPORTS")?;
        let mut has_args = false;
        for arg in self.iter.clone() {
            has_args = true;
            self.write_supported(f, arg.as_ref())?;
        }
        if !has_args {
            self.write_all_supported(f)?;
        }
        write!(f, ">\r\n")
    }
}
