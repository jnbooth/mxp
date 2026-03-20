use std::fmt;

use crate::arguments::{ArgumentScanner, Arguments, ExpectArg as _};
use crate::color::RgbColor;
use crate::keyword::LineTagKeyword;
use crate::line::Mode;
use crate::{Error, ErrorKind};

/// Parsed representation of a line tag definition from the server, in the form of
/// `<!TAG {index} ...>`.
///
/// Full definition:
///
/// ```xml
/// <!TAG
///     Index
///     [WINDOW=string]
///     [FORE=color]
///     [BACK=color]
///     [GAG]
///     [ENABLE]
///     [DISABLE]
/// >
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LineTagDefinition<'a> {
    /// Tag number (20-99) to change.
    pub index: Mode,
    /// Window to redirect the text to.
    pub window: Option<&'a str>,
    /// Text should be gagged from the main MUD window.
    pub gag: Option<bool>,
    /// Text color.
    pub fore: Option<RgbColor>,
    /// Background color of the text.
    pub back: Option<RgbColor>,
    /// If `Some(true)`, activates the line tag. If `Some(false)`, deactivates it.
    pub enable: Option<bool>,
}

impl fmt::Display for LineTagDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Self {
            index,
            window,
            gag,
            fore,
            back,
            enable,
        } = self;
        write!(f, "<!TAG {index}")?;
        if let Some(window) = window {
            write!(f, " WINDOWNAME=\"{window}\"")?;
        }
        if gag == Some(true) {
            f.write_str(" GAG")?;
        }
        if let Some(fore) = fore {
            write!(f, " FORE={fore}")?;
        }
        if let Some(back) = back {
            write!(f, " BACK={back}")?;
        }
        match enable {
            Some(true) => f.write_str(" ENABLE>"),
            Some(false) => f.write_str(" DISABLE>"),
            None => f.write_str(">"),
        }
    }
}

impl<'a> LineTagDefinition<'a> {
    pub(super) fn parse(source: &'a str) -> crate::Result<Self> {
        let args = Arguments::parse(source)?;
        let mut scanner = args.scan(()).with_keywords();
        let index = Mode(scanner.get_next().expect_number()?.expect_some("Tag")?);
        if !index.is_user_defined() {
            return Err(Error::new(index.to_string(), ErrorKind::IllegalLineTag));
        }
        let window = scanner.get_named("windowname").copied();
        let fore = scanner.get_named("fore").expect_color()?;
        let back = scanner.get_named("back").expect_color()?;
        let keywords = scanner.into_keywords()?;
        let gag = if keywords.contains(LineTagKeyword::Gag) {
            Some(true)
        } else {
            None
        };
        let enable = if keywords.contains(LineTagKeyword::Disable) {
            Some(false)
        } else if keywords.contains(LineTagKeyword::Enable) {
            Some(true)
        } else {
            None
        };
        Ok(Self {
            index,
            window,
            gag,
            fore,
            back,
            enable,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_linetag() {
        let def = LineTagDefinition {
            index: Mode(30),
            window: Some("_top"),
            gag: Some(true),
            fore: Some(RgbColor::hex(0x123456)),
            back: Some(RgbColor::hex(0x789abc)),
            enable: Some(false),
        };
        assert_eq!(
            def.to_string(),
            "<!TAG 30 WINDOWNAME=\"_top\" GAG FORE=#123456 BACK=#789abc DISABLE>"
        );
    }
}
