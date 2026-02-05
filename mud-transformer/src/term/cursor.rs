#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CursorEffect {
    /// CUU (Cursor Up)
    Up(u16),
    /// CUD (Cursor Down)
    Down(u16),
    /// CUF (Cursor Forward)
    Forward(u16),
    /// CUB (Cursor Backward)
    Back(u16),

    /// CNL (Cursor Next Line)
    NextLine(u16),
    /// CPL (Cursor Previous Line)
    PreviousLine(u16),

    /// NP (Next Page)
    NextPage(usize),
    /// PP (Preceding Page)
    PrecedingPage(usize),
    /// PPB (Page Position Backward)
    PageBackward(usize),
    /// PPR (Page Position Relative)
    PageForward(usize),
    /// PPA (Page Position Absolute)
    PageAbsolute(usize),

    /// CHT (Cursor Horizontal Forward Tabulation)
    TabForward(u16),
    /// CBT (Cursor Backward Tabulation)
    TabBack(u16),

    /// CUP (Cursor Position),
    /// HVP (Horizontal and Vertical Position)
    Position { row: u16, column: u16 },

    /// HPA (Horizontal Position Absolute)
    ColumnAbsolute(u16),
    /// HPR (Horizontal Position Relative)
    ColumnRelative(u16),
    /// VPA (Vertical Line Position Absolute)
    RowAbsolute(u16),
    /// VPR (Vertical Position Relative)
    RowRelative(u16),

    /// CHA (Cursor Horizontal Absolute)
    HorizontalAbsolute(u16),

    /// SU (Pan Down)
    ScrollUp(u16),
    /// SD (Pan Up)
    ScrollDown(u16),

    /// IND (Index)
    Index,
    /// RI (Reverse Index)
    ReverseIndex,

    /// SCOSC (Save Current Cursor Position)
    Save,
    /// SCORC (Restore Saved Cursor Position)
    Restore,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EraseRange {
    #[default]
    AfterCursor,
    BeforeCursor,
    Full,
}

impl EraseRange {
    pub(crate) fn from_code(code: Option<u16>) -> Option<Self> {
        match code {
            None | Some(0) => Some(Self::AfterCursor),
            Some(1) => Some(Self::BeforeCursor),
            Some(2) => Some(Self::Full),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EraseTarget {
    Line,
    Display,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CursorStyle {
    #[default]
    BlinkBlock,
    SteadyBlock,
    BlinkUnderline,
    SteadyUnderline,
}

impl CursorStyle {
    pub const fn code(self) -> u8 {
        self as u8
    }

    pub(crate) const fn from_code(code: Option<u16>) -> Option<Self> {
        match code {
            None | Some(0 | 1) => Some(Self::BlinkBlock),
            Some(2) => Some(Self::SteadyBlock),
            Some(3) => Some(Self::BlinkUnderline),
            Some(4) => Some(Self::SteadyUnderline),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HighlightTracking {
    pub func: bool,
    pub start_x: u16,
    pub start_y: u16,
    pub first_row: u16,
    pub last_row: u16,
}
