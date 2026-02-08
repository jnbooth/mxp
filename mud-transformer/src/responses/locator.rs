use std::fmt;

use flagset::{FlagSet, flags};
use mxp::escape::ansi::CSI;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LocatorReportEvent {
    /// Locator unavailable - no other parameters sent
    Unavailable = 0,
    /// Request - xterm received a DECRQLP
    Request,
    LeftDown,
    LeftUp,
    MiddleDown,
    MiddleUp,
    RightDown,
    RightUp,
    M4Down,
    M4Up,
    /// Locator outside filter rectangle
    Outside,
}

flags! {
    pub enum LocatorReportButton: u8 {
        Right,
        Middle,
        Left,
        M4,
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LocatorReport {
    pub event: LocatorReportEvent,
    pub button: FlagSet<LocatorReportButton>,
    pub row: u16,
    pub column: u16,
    pub page: usize,
}

impl fmt::Display for LocatorReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            event,
            button,
            row,
            column,
            page,
        } = self;
        let event = *event as u8;
        let button = button.bits();
        if self.event == LocatorReportEvent::Unavailable {
            return write!(f, "{CSI}{event}");
        }
        write!(f, "{CSI}{event};{button};{row};{column};{page}")
    }
}
