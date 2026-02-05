use std::time::Duration;

use bytes::Bytes;
use bytestring::ByteString;
use mxp::RgbColor;

use super::OutputFragment;
use crate::term;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ControlFragment {
    Backspace,
    Beep,
    /// CR (Carriage Return)
    CarriageReturn,
    Clear,
    /// TBC (Tab Clear)
    ClearTab,
    /// TBC (Tab Clear)
    ClearTabs,
    ControlString(u8, Bytes),
    /// DECCRA (Copy Rectangular Area)
    CopyRect {
        rect: term::Rect,
        source: usize,
        row: u16,
        column: u16,
        target: usize,
    },
    Cursor(term::CursorEffect),
    DEC(term::Dec),
    /// DCH (Delete Character)
    DeleteCharacters(usize),
    /// DECDC (Delete Column)
    DeleteColumns(usize),
    /// DL (Delete Line)
    DeleteLines(usize),
    /// DSR (Device Status Reports)
    DeviceStatusReport(term::DeviceStatus),
    /// DECSED (Selective Erase in Display),
    /// DECSEL (Selective Erase in Line),
    /// ED (Erase in Display),
    /// EL (Erase in Line)
    Erase {
        target: term::EraseTarget,
        range: term::EraseRange,
        selective: bool,
    },
    EraseCharacter,
    /// ECH (Erase Character)
    EraseCharacters(usize),
    /// DECERA (Erase Rectangular Area),
    /// DECSERA (Selective Erase Rectangular Area)
    EraseRect {
        rect: term::Rect,
        selective: bool,
    },
    /// DECFRA (Fill Rectangular Area)
    FillRect {
        rect: term::Rect,
        fill_char: u8,
    },
    /// DECEFR (Enable Filter Rectangle)
    FilterRect {
        rect: term::Rect,
    },
    /// DECFNK (Function Key)
    FunctionKey {
        keystroke: u8,
        modifiers: u8,
    },
    /// SPA (Start of Guarded Area)
    GuardedAreaStart,
    /// EPA (End of Guarded Area)
    GuardedAreaEnd,
    /// DECSLRM (Set Left and Right Margins)
    HMargins {
        left: Option<u16>,
        right: Option<u16>,
    },
    /// DECIC (Insert Column)
    InsertColumns(usize),
    /// IL (Insert Line)
    InsertLines(usize),
    /// ICH (Insert Character)
    InsertSpaces(usize),
    ManipulateSelection(term::SelectionData, ByteString),
    /// MC (Media Copy)
    MediaCopy(term::PrintFunction),
    ModeRequest(term::Mode),
    ModeRestore(term::Mode),
    ModeSave(term::Mode),
    /// SM (Set Mode),
    /// RM (Reset Mode)
    ModeSet(term::Mode, bool),
    /// NEL (Next Line)
    ///
    /// Rarely used. Not to be confused with [`CursorEffect::NextLine`].
    NextLine,
    QueryKeyFormat(u8),
    Repeat(usize),
    Request(term::AttributeRequest),
    ResetKeyFormat,
    /// DECSR (Secure Reset),
    /// DECSTR (Soft Terminal Reset),
    /// RIS (Reset to Initial State)
    ResetTerminal(term::Reset),
    /// DECRSPS (Restore Presentation State)
    RestoreTabStops(Vec<u16>),
    /// DECRARA (Reverse Attributes in Rectangular Area)
    ReverseRectAttribute {
        rect: term::Rect,
        attribute: term::ReverseVisualCharacterAttribute,
    },
    /// DECSLE (Select Locator Events)
    SelectLocatorEvents {
        on_press: bool,
        on_release: bool,
    },
    /// DECSASD (Select Active Status Display)
    ///
    /// - **true:** Selects the status line. The terminal sends data to the status line only.
    /// - **false:** Selects the main display. The terminal sends data to the main display only.
    SelectStatusLine(bool),
    /// DECSACE (Select Attribute Change Extent)
    ///
    /// - **true:** DECCARA and DECRARA affect all character positions in the rectangular area. The DECCARA or DECRARA command specifies the top-left and bottom-right corners.
    /// - **false:** DECCARA or DECRARA affect the stream of character positions that begins with the first position specified in the DECCARA or DECRARA command, and ends with the second character position specified.
    SetAttributeChangeExtent(bool),
    SetCharacterProtection(bool),
    /// DECSCPP (Select 80 or 132 Columns per Page)
    SetColumns(u16),
    /// DECSDDT (Select Disconnect Delay Time)
    SetDisconnectDelay(Duration),
    SetDynamicColor(term::DynamicColor, RgbColor),
    SetFont(ByteString),
    SetIconLabel(ByteString),
    /// DECSKCV (Set Key Click Volume)
    SetKeyClickVolume(u8),
    SetKeyFormat {
        param: u8,
        value: Option<u16>,
    },
    /// DECSMBV (Set Margin Bell Volume)
    SetMarginVolume(u8),
    /// Locks memory above the cursor.
    SetMemoryLock(bool),
    /// DECLL (Load LEDs)
    SetLed(term::KeyboardLed, bool),
    SetLocator(term::LocatorReporting, term::LocatorUnit),
    /// DECCARA (Change Attributes in Rectangular Area)
    SetRectAttribute {
        rect: term::Rect,
        attribute: term::VisualCharacterAttribute,
    },
    /// DECSRFR (Select Refresh Rate)
    SetRefreshRate(term::RefreshRate),
    /// DECSNLS (Set Lines Per Screen)
    SetRows(u16),
    /// DECSSCLS (Set Scroll Speed)
    SetScrollSpeed(u8),
    SetShiftEscape(bool),
    /// DECSSDT (Select Status Display (Line) Type)
    SetStatusDisplay(term::StatusDisplayType),
    /// HTS (Horizontal Tab Set)
    SetTabStop,
    SetTitle(ByteString),
    /// "prop=value", or just "prop" to delete the property
    SetXProperty(ByteString),
    /// DECSWBV (Set Warning Bell Volume)
    SetWarningVolume(u8),
    /// DECSCUSR (Set Cursor Style)
    StyleCursor(term::CursorStyle),
    /// DECLTOD (Load Time of Day)
    TimeOfDay {
        hour: u8,
        minute: u8,
    },
    Track(term::HighlightTracking),
    Window(term::WindowOp),
    /// VT (Vertical Tab)
    VerticalTab,
    /// DECSTBM (Set Top and Bottom Margins)
    VMargins {
        top: Option<u16>,
        bottom: Option<u16>,
    },
}

impl ControlFragment {
    pub(super) const fn should_flush(&self) -> bool {
        !matches!(
            self,
            Self::Beep | Self::SetMarginVolume(_) | Self::SetWarningVolume(_) | Self::SetTitle(_),
        )
    }
}

impl From<ControlFragment> for OutputFragment {
    fn from(value: ControlFragment) -> Self {
        Self::Control(value)
    }
}

impl From<term::AttributeRequest> for ControlFragment {
    fn from(value: term::AttributeRequest) -> Self {
        Self::Request(value)
    }
}

impl From<term::AttributeRequest> for OutputFragment {
    fn from(value: term::AttributeRequest) -> Self {
        Self::Control(ControlFragment::Request(value))
    }
}

impl From<term::CursorEffect> for ControlFragment {
    fn from(value: term::CursorEffect) -> Self {
        Self::Cursor(value)
    }
}

impl From<term::CursorEffect> for OutputFragment {
    fn from(value: term::CursorEffect) -> Self {
        Self::Control(ControlFragment::Cursor(value))
    }
}

impl From<term::Dec> for ControlFragment {
    fn from(value: term::Dec) -> Self {
        Self::DEC(value)
    }
}

impl From<term::Dec> for OutputFragment {
    fn from(value: term::Dec) -> Self {
        Self::Control(ControlFragment::DEC(value))
    }
}

impl From<term::HighlightTracking> for ControlFragment {
    fn from(value: term::HighlightTracking) -> Self {
        Self::Track(value)
    }
}

impl From<term::HighlightTracking> for OutputFragment {
    fn from(value: term::HighlightTracking) -> Self {
        Self::Control(ControlFragment::Track(value))
    }
}

impl From<term::WindowOp> for ControlFragment {
    fn from(value: term::WindowOp) -> Self {
        Self::Window(value)
    }
}

impl From<term::WindowOp> for OutputFragment {
    fn from(value: term::WindowOp) -> Self {
        Self::Control(ControlFragment::Window(value))
    }
}
