use std::fmt;

use crate::escape::ansi::CSI;

/// Formats a DSR-OK response.
#[derive(Copy, Clone, Debug)]
pub struct OkReport;

impl fmt::Display for OkReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{CSI}0n")
    }
}

/// Formats a DSR-CPR response.
#[derive(Copy, Clone, Debug)]
pub struct CursorPositionReport {
    /// Indicates what line the cursor is on.
    pub row: u16,
    /// Indicates what column the cursor is at.
    pub column: u16,
}

impl fmt::Display for CursorPositionReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { row, column } = self;
        write!(f, "{CSI}{row};{column}R")
    }
}

/// Formats a DSR-XCPR response.
#[derive(Copy, Clone, Debug)]
pub struct ExtendedCursorPositionReport {
    /// Indicates what line the cursor is on.
    pub row: u16,
    /// Indicates what column the cursor is at.
    pub column: u16,
    /// Indicates what page the cursor is on.
    pub page: usize,
}

impl fmt::Display for ExtendedCursorPositionReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { row, column, page } = self;
        write!(f, "{CSI}{row};{column};{page}R")
    }
}

/// Formats a DSR-DIR response.
#[derive(Copy, Clone, Debug)]
pub enum DataIntegrityReport {
    Ready = 70,
    Malfunction = 71,
    Unreported = 73,
}

impl fmt::Display for DataIntegrityReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = *self as u8;
        write!(f, "{CSI}?{code}n")
    }
}

/// Formats a DSR-KBD response.
#[derive(Copy, Clone, Debug)]
pub struct KeyboardReport {
    pub language: u8,
    pub keyboard_status: u8,
    pub keyboard_type: u8,
}

impl KeyboardReport {
    pub const KEYBOARD_STATUS_READY: u8 = 0;
    pub const KEYBOARD_STATUS_UNDETECTED: u8 = 3;
    pub const KEYBOARD_STATUS_BUSY: u8 = 8;

    pub const KEYBOARD_TYPE_LK450: u8 = 4;
    pub const KEYBOARD_TYPE_PCXAL: u8 = 5;
}

/// Formats a DSR-OS response.
#[derive(Copy, Clone, Debug)]
pub enum OperatingStatusReport {
    Good = 0,
    Malfunction = 3,
}

impl fmt::Display for OperatingStatusReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = *self as u8;
        write!(f, "{CSI}{code}n")
    }
}
