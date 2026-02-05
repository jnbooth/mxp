use std::fmt;

use mxp::escape::ansi::CSI;

#[derive(Copy, Clone, Debug)]
pub struct PrimaryAttributeReport;

impl fmt::Display for PrimaryAttributeReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{CSI}?1;2c")
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SecondaryAttributeReport;

impl fmt::Display for SecondaryAttributeReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{CSI}?1;95;0c")
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TerminalParamsReport;

impl fmt::Display for TerminalParamsReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{CSI}1;1;112;112;1;0x")
    }
}
