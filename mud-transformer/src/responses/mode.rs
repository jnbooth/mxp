use std::fmt;

use mxp::escape::ansi::CSI;

use crate::term::Mode;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModeReportSetting {
    NotRecognized,
    Set,
    Reset,
    PermanentlySet,
    PermanentlyReset,
}

#[derive(Copy, Clone, Debug)]
pub struct ModeReport {
    pub mode: Mode,
    pub setting: ModeReportSetting,
}

impl fmt::Display for ModeReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { mode, setting } = self;
        let setting = *setting as u8;
        write!(f, "{CSI}{mode};{setting}$y")
    }
}
