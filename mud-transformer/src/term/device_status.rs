use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeviceStatus {
    Standard(u8),
    Private(u8),
}

impl DeviceStatus {
    /// DSR-OS (Operating Status)
    pub const OPERATING_STATUS: Self = Self::Standard(5);
    /// DSR-CPR (Cursor Position Report)
    pub const CURSOR_POSITION: Self = Self::Standard(6);

    /// DSR-XCPR (Extended Cursor Position)
    pub const EXTENDED_CURSOR_POSITION: Self = Self::Private(6);
    /// DSR-PP (Printer Port)
    pub const PRINTER_PORT: Self = Self::Private(15);
    /// DSR-UDK (User-Defined Keys)
    pub const USER_DEFINED_KEYS: Self = Self::Private(25);
    /// DSR-KBD (Keyboard)
    pub const KEYBOARD: Self = Self::Private(26);
    /// DSR-DIR (Data Integrity Report)
    pub const DATA_INTEGRITY: Self = Self::Private(75);
    /// DSR-MSR (Macro Space Report)
    pub const MACRO_SPACE: Self = Self::Private(62);
    /// DSR-CKSR (Memory Checksum)
    pub const MEMORY_CHECKSUM: Self = Self::Private(63);

    pub const fn new(code: u8, private: bool) -> Self {
        if private {
            Self::Private(code)
        } else {
            Self::Standard(code)
        }
    }

    pub const fn code(self) -> u8 {
        match self {
            Self::Standard(code) | Self::Private(code) => code,
        }
    }

    pub const fn private(&self) -> bool {
        matches!(self, Self::Private(_))
    }
}

impl fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Standard(code) => write!(f, "{code}"),
            Self::Private(code) => write!(f, "?{code}"),
        }
    }
}
