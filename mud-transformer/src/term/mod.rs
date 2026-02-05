mod character;
pub use character::{ReverseVisualCharacterAttribute, VisualCharacterAttribute};

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

mod selection;
pub use selection::SelectionData;

mod window;
pub use window::{RefreshRate, WindowOp};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AttributeRequest {
    /// DECCIR (Cursor Information Report) with [`CursorInformationReport`](crate::responses::CursorInformationReport)
    CursorInformation,
    /// DECRPDE (Report Displayed Extent) with [`DisplayedExtentReport`](crate::responses::DisplayedExtentReport)
    DisplayedExtent,
    /// DECLRP (Report Locator Position) with [`LocatorPositionReport`](crate::responses::LocatorPositionReport)
    LocatorPosition,
    /// DECRQUPSS
    PreferredSupplementalSet,
    /// DECTABSR (Tab Stop Report) with [`TabStopReport`](crate::responses::TabStopReport)
    TabStop,
    /// DECTSR (Terminal State Report)
    TerminalState,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Dec {
    /// DECSC (Save Cursor)
    SaveCursor,
    /// DECRC (Restore Cursor)
    RestoreCursor,

    /// DECKPAM (Keyboard Application Mode)
    ApplicationKeypad,
    /// DECKPNM (Keypad Numeric Mode)
    NormalKeypad,

    /// DECSCSA (Select Character Protection Attribute)
    CharacterProtection,

    /// DECST8C (Set Tab at Every 8 Columns)
    Tab8Columns,

    /// DECSWL (Single-Width, Single-Height Line)
    SingleWidthLine,
    /// DECDWL (Double-Width, Single-Height Line)
    DoubleWidthLine,
    /// DECDHL (Double-Width, Double-Height Line) Top Half
    DoubleHeightLineTop,
    /// DECDHL (Double-Width, Double-Height Line) Bottom Half
    DoubleHeightLineBottom,

    /// DECFI (Forward Index)
    ForwardIndex,
    /// DECBI (Back Index)
    BackIndex,

    /// DECALN (Screen Alignment Pattern)
    ScreenAlignmentTest,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyboardLed {
    All,
    NumLock,
    CapsLock,
    ScrollLock,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rect {
    pub top: Option<u16>,
    pub left: Option<u16>,
    pub bottom: Option<u16>,
    pub right: Option<u16>,
}

impl Rect {
    pub const fn new() -> Self {
        Self {
            top: None,
            left: None,
            bottom: None,
            right: None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Reset {
    /// DECSTR (Soft Terminal Reset)
    Soft,
    /// RIS (Reset to Initial State)
    Hard,
    /// DECSR (Secure Reset)
    Secure,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
