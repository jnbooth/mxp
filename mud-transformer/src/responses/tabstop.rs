use std::fmt;

use mxp::escape::ansi::{DCS, ST};

use crate::output::{BufferedOutput, ControlFragment};

/// Formats a DECTABSR report.
#[derive(Copy, Clone, Debug)]
pub struct TabStopReport<T> {
    pub stops: T,
}

impl<T: AsRef<[u16]>> fmt::Display for TabStopReport<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut iter = self.stops.as_ref().iter();
        let Some(first) = iter.next() else {
            return write!(f, "{DCS}2$u{ST}");
        };
        write!(f, "{DCS}2$u{first}")?;
        for stop in iter {
            write!(f, "/{stop}")?;
        }
        write!(f, "{ST}")
    }
}

impl<T> TabStopReport<T>
where
    T: FromIterator<u16>,
{
    pub(crate) fn decode(s: &str) -> Self {
        let stops = s.split('/').map_while(|s| s.parse().ok()).collect();
        Self { stops }
    }
}

impl TabStopReport<Vec<u16>> {
    pub(crate) fn restore(self, output: &mut BufferedOutput) {
        output.append(ControlFragment::RestoreTabStops(self.stops));
    }
}
