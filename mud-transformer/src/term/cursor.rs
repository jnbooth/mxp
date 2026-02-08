#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    NextPage(u16),
    /// PP (Preceding Page)
    PrecedingPage(u16),
    /// PPB (Page Position Backward)
    PageBackward(u16),
    /// PPR (Page Position Relative)
    PageForward(u16),
    /// PPA (Page Position Absolute)
    PageAbsolute(u16),

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
    /// DECFI (Forward Index)
    ForwardIndex,
    /// DECBI (Back Index)
    BackIndex,

    /// `Save { dec: false }`: SCOSC (Save Current Cursor Position)
    ///
    /// `Save { dec: true }`: DECSC (Save Cursor)
    ///
    /// DECSC is supposed to save more information. See https://vt100.net/docs/vt510-rm/DECSC.html
    Save { dec: bool },
    /// `Save { dec: false }`: SCORC (Restore Saved Cursor Position)
    ///
    /// `Save { dec: true }`: DECRC (Restore Cursor)
    ///
    /// These correspond to the states of [`CursorEffect::Save`].
    Restore { dec: bool },
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum EraseRange {
    #[default]
    AfterCursor = 0,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EraseTarget {
    Display = b'J' as isize,
    Line = b'K' as isize,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum CursorStyle {
    #[default]
    BlinkBlock = 1,
    SteadyBlock,
    BlinkUnderline,
    SteadyUnderline,
}

impl CursorStyle {
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct HighlightTracking {
    pub func: bool,
    pub start_x: u16,
    pub start_y: u16,
    pub first_row: u16,
    pub last_row: u16,
}
