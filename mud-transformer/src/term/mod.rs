mod color;
pub use color::DynamicColor;
pub(crate) use color::{TermColor, XTermPalette};

mod cursor;
pub use cursor::{CursorEffect, CursorStyle, EraseRange, EraseTarget, HighlightTracking};

mod device_status;
pub use device_status::DeviceStatus;

mod locator;
pub use locator::{LocatorReporting, LocatorUnit};

mod mode;
pub use mode::Mode;

mod print_function;
pub use print_function::PrintFunction;

mod rect;
pub use rect::{Rect, RectEffect, ReverseVisualCharacterAttribute, VisualCharacterAttribute};

mod selection;
pub use selection::SelectionData;

mod window;
pub use window::{RefreshRate, WindowOp};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AttributeRequest {
    /// DECCIR (Cursor Information Report) with [`CursorInformationReport`](crate::responses::CursorInformationReport)
    CursorInformation,
    /// DECRPDE (Report Displayed Extent) with [`DisplayedExtentReport`](crate::responses::DisplayedExtentReport)
    DisplayedExtent,
    /// DECLRP (Report Locator Position) with [`LocatorPositionReport`](crate::responses::LocatorPositionReport)
    LocatorPosition,
    /// DECTABSR (Tab Stop Report) with [`TabStopReport`](crate::responses::TabStopReport)
    TabStop,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ControlStringType {
    /// SOS (Start of String)
    Sos = b'X' as isize,
    /// PM (Private Message)
    Pm = b'^' as isize,
    /// APC (Application Program Command)
    Apc = b'_' as isize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Line {
    /// DECDHL (Double-Width, Double-Height Line) Top Half
    DoubleHeightTop = 3,
    /// DECDHL (Double-Width, Double-Height Line) Bottom Half
    DoubleHeightBottom,
    /// DECSWL (Single-Width, Single-Height Line)
    SingleWidth,
    /// DECDWL (Double-Width, Single-Height Line)
    DoubleWidth,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyboardLed {
    All,
    NumLock,
    CapsLock,
    ScrollLock,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reset {
    /// DECSTR (Soft Terminal Reset)
    Soft,
    /// RIS (Reset to Initial State)
    Hard,
    /// DECSR (Secure Reset)
    Secure,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum StatusDisplayType {
    #[default]
    Hidden,
    Indicator,
    HostWritable,
}

impl StatusDisplayType {
    pub(crate) const fn from_code(code: Option<u16>) -> Option<Self> {
        match code {
            None | Some(0) => Some(Self::Hidden),
            Some(1) => Some(Self::Indicator),
            Some(2) => Some(Self::HostWritable),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TabEffect {
    /// TBC (Tab Clear)
    ClearAtCursor,
    /// TBC (Tab Clear)
    ClearAll,
    /// DECRSPS (Restore Presentation State)
    RestoreStops(Vec<u16>),
    /// DECST8C (Set Tab at Every 8 Columns)
    SetEvery8Columns,
    /// HTS (Horizontal Tab Set)
    SetStop,
}
