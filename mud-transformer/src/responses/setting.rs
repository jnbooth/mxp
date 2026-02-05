use std::fmt;

use mxp::escape::ansi::{DCS, ST};
use mxp::{FlagSet, RgbColor};

use crate::output::TextStyle;
use crate::term::TermColor;

const PRE: &str = "\x1BP0$r";

struct IfSome<T>(Option<T>);
impl<T> fmt::Display for IfSome<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Some(x) => x.fmt(f),
            None => Ok(()),
        }
    }
}

/// Formats a DECRPSS response.
#[derive(Copy, Clone, Debug)]
pub(crate) struct SgrReport {
    pub flags: FlagSet<TextStyle>,
    pub foreground: TermColor,
    pub background: TermColor,
}

impl fmt::Display for SgrReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            flags,
            foreground,
            background,
        } = self;
        write!(f, "{PRE}0")?;
        for ansi in flags.into_iter().filter_map(TextStyle::ansi) {
            write!(f, ";{ansi}")?;
        }
        match foreground {
            TermColor::Unset => (),
            TermColor::Ansi(i) => write!(f, ";38;5;{i}")?,
            TermColor::Rgb(RgbColor { r, g, b }) => write!(f, ";38;2;{r};{g};{b}")?,
        }
        match background {
            TermColor::Unset => (),
            TermColor::Ansi(i) => write!(f, ";48;5;{i}")?,
            TermColor::Rgb(RgbColor { r, g, b }) => write!(f, ";48;2;{r};{g};{b}")?,
        }
        write!(f, "m{ST}")
    }
}

/// Formats a DECRPSS response for DECSTBM.
#[derive(Copy, Clone, Debug)]
pub(crate) struct VMarginsReport {
    pub top: Option<u16>,
    pub bottom: Option<u16>,
}

impl fmt::Display for VMarginsReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let top = IfSome(self.top);
        let bottom = IfSome(self.bottom);
        write!(f, "{PRE}{top};{bottom}s")
    }
}

/// Formats a DECRPSS response for DECSLRM.
#[derive(Copy, Clone, Debug)]
pub(crate) struct HMarginsReport {
    pub left: Option<u16>,
    pub right: Option<u16>,
}

impl fmt::Display for HMarginsReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let left = IfSome(self.left);
        let right = IfSome(self.right);
        write!(f, "{PRE}{left};{right}s")
    }
}

/// Formats a DECRPSS response.
#[derive(Copy, Clone, Debug)]
pub(crate) struct UnknownSettingReport;

impl fmt::Display for UnknownSettingReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{DCS}1$r{ST}")
    }
}
