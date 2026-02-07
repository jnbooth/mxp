use std::fmt;

use mxp::escape::ansi::{DCS, ST};
use mxp::{FlagSet, RgbColor};

use crate::output::TextStyle;
use crate::term::TermColor;

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
        write!(f, "\x1BP0$r0")?;
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

/// Formats a DECRPSS response.
#[derive(Copy, Clone, Debug)]
pub(crate) struct UnknownSettingReport;

impl fmt::Display for UnknownSettingReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{DCS}1$r{ST}")
    }
}
