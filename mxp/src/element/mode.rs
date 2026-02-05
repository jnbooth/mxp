use std::fmt;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mode(pub u8);

impl Mode {
    /// Only MXP commands in the "open" category are allowed.
    pub const OPEN: Self = Self(0);
    /// All tags and commands in MXP are allowed within the line.
    pub const SECURE: Self = Self(1);
    /// No MXP or HTML commands are allowed in the line. The line is not parsed for any tags at all.
    pub const LOCKED: Self = Self(2);
    /// Close all open tags.
    pub const RESET: Self = Self(3);
    /// Next tag is secure only.
    pub const SECURE_ONCE: Self = Self(4);
    /// Open mode until mode change.
    pub const PERM_OPEN: Self = Self(5);
    /// Secure mode until mode change.
    pub const PERM_SECURE: Self = Self(6);
    /// Locked mode until mode change.
    pub const PERM_LOCKED: Self = Self(7);

    pub const USER_DEFINED_MIN: Self = Self(20);
    pub const USER_DEFINED_MAX: Self = Self(99);
}

impl Mode {
    pub const fn is_open(self) -> bool {
        matches!(self, Self::OPEN | Self::PERM_OPEN)
    }

    pub const fn is_secure(self) -> bool {
        matches!(self, Self::SECURE | Self::SECURE_ONCE | Self::PERM_SECURE)
    }

    pub const fn is_mxp(self) -> bool {
        matches!(
            self,
            Self::OPEN | Self::PERM_OPEN | Self::SECURE | Self::SECURE_ONCE | Self::PERM_SECURE
        )
    }

    pub const fn is_user_defined(self) -> bool {
        self.0 >= Self::USER_DEFINED_MIN.0 && self.0 <= Self::USER_DEFINED_MAX.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ModeRangeError;

impl fmt::Display for ModeRangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("MXP mode must be between 0 and 99")
    }
}

macro_rules! impl_try_from_unsigned {
    ($t:ty) => {
        impl TryFrom<$t> for Mode {
            type Error = ModeRangeError;

            fn try_from(value: $t) -> Result<Self, Self::Error> {
                if value <= 99 {
                    #[allow(clippy::cast_possible_truncation)]
                    Ok(Self(value as u8))
                } else {
                    Err(ModeRangeError)
                }
            }
        }
    };
}
macro_rules! impl_try_from_signed {
    ($t:ty) => {
        impl TryFrom<$t> for Mode {
            type Error = ModeRangeError;

            fn try_from(value: $t) -> Result<Self, Self::Error> {
                #[allow(clippy::manual_range_contains)]
                if value >= 0 && value <= 99 {
                    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                    Ok(Self(value as u8))
                } else {
                    Err(ModeRangeError)
                }
            }
        }
    };
}

impl_try_from_unsigned!(u8);
impl_try_from_unsigned!(u16);
impl_try_from_unsigned!(u32);
impl_try_from_unsigned!(u64);
impl_try_from_unsigned!(usize);
impl_try_from_signed!(i8);
impl_try_from_signed!(i16);
impl_try_from_signed!(i32);
impl_try_from_signed!(i64);
impl_try_from_signed!(isize);
