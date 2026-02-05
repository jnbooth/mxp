#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReverseVisualCharacterAttribute {
    #[default]
    All = 0,
    Bold = 1,
    Underline = 4,
    Blink = 5,
    Inverse = 7,
}

impl ReverseVisualCharacterAttribute {
    pub const fn code(self) -> u8 {
        self as u8
    }

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
