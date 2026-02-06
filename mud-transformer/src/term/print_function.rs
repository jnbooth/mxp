use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PrintFunction {
    Standard(u8),
    Private(u8),
}

impl PrintFunction {
    /// Prints the page that has the cursor.
    pub const PRINT_PAGE: Self = Self::Standard(0);
    /// Sends screen data through host port.
    pub const SEND: Self = Self::Standard(2);
    /// Turns off printer controller mode.
    pub const DISABLE_PRINTER_CONTROLLER: Self = Self::Standard(4);
    /// Turns on printer controller mode.
    pub const ENABLE_PRINTER_CONTROLLER: Self = Self::Standard(5);
    /// Disables a printer-to-host session.
    /// Functionally equivalent to [`DISABLE_PORT_TO_HOST`](Self::DISABLE_PORT_TO_HOST).
    pub const DISABLE_PRINTER_TO_HOST: Self = Self::Standard(6);
    /// Enables a printer-to-host session.
    /// Functionally equivalent to [`ENABLE_PORT_TO_HOST`](Self::DISABLE_PORT_TO_HOST).
    pub const ENABLE_PRINTER_TO_HOST: Self = Self::Standard(7);

    /// Prints the line that has the cursor.
    pub const PRINT_LINE: Self = Self::Private(1);
    /// Turns off autoprint mode.
    pub const DISABLE_AUTOPRINT: Self = Self::Private(4);
    /// Turns on autoprint mode.
    pub const ENABLE_AUTOPRINT: Self = Self::Private(5);
    /// Disables communication from the printer port to the host.
    /// Functionally equivalent to [`DISABLE_PRINTER_TO_HOST`](Self::DISABLE_PRINTER_TO_HOST).
    pub const DISABLE_PORT_TO_HOST: Self = Self::Private(8);
    /// Enables communication from the printer port to the host.
    /// Functionally equivalent to [`ENABLE_PRINTER_TO_HOST`](Self::ENABLE_PRINTER_TO_HOST).
    pub const ENABLE_PORT_TO_HOST: Self = Self::Private(9);
    /// Prints the data on the screen.
    pub const PRINT_SCREEN: Self = Self::Private(10);
    /// Prints all pages in page memory.
    pub const PRINT_ALL: Self = Self::Private(11);

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

impl fmt::Display for PrintFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Standard(code) => write!(f, "{code}"),
            Self::Private(code) => write!(f, "?{code}"),
        }
    }
}
