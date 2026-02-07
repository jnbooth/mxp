use crate::ControlFragment;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RectEffect {
    /// DECCRA (Copy Rectangular Area)
    Copy {
        /// Source page
        source: u16,
        /// Target row
        row: u16,
        /// Target column
        column: u16,
        /// Target page
        target: u16,
    },
    /// DECERA (Erase Rectangular Area),
    /// DECSERA (Selective Erase Rectangular Area)
    Erase { selective: bool },
    /// DECFRA (Fill Rectangular Area)
    Fill { fill_char: u8 },
    /// DECEFR (Enable Filter Rectangle)
    Filter,
    /// DECRARA (Reverse Attributes in Rectangular Area)
    ReverseAttributes(super::ReverseVisualCharacterAttribute),
    /// DECCARA (Change Attributes in Rectangular Area)
    SetAttributes(super::VisualCharacterAttribute),
}

impl RectEffect {
    pub const fn with(self, rect: Rect) -> ControlFragment {
        ControlFragment::Rect(rect, self)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum VisualCharacterAttribute {
    #[default]
    Reset = 0,
    Bold = 1,
    Underline = 4,
    Blink = 5,
    Inverse = 7,
    NoBold = 22,
    NoUnderline = 24,
    NoBlink = 25,
    NoInverse = 27,
}

impl VisualCharacterAttribute {
    pub const fn code(self) -> u8 {
        self as u8
    }

    pub(crate) const fn from_code(code: u16) -> Option<Self> {
        match code {
            0 => Some(Self::Reset),
            1 => Some(Self::Bold),
            4 => Some(Self::Underline),
            5 => Some(Self::Blink),
            7 => Some(Self::Inverse),
            22 => Some(Self::NoBold),
            24 => Some(Self::NoUnderline),
            25 => Some(Self::NoBlink),
            27 => Some(Self::NoInverse),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ReverseVisualCharacterAttribute {
    #[default]
    All = 0,
    Bold = 1,
    Underline = 4,
    Blink = 5,
    Inverse = 7,
}

impl ReverseVisualCharacterAttribute {
    pub(crate) const fn from_code(code: u16) -> Option<Self> {
        match code {
            0 => Some(Self::All),
            1 => Some(Self::Bold),
            4 => Some(Self::Underline),
            5 => Some(Self::Blink),
            7 => Some(Self::Inverse),
            _ => None,
        }
    }
}
