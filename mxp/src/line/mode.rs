use std::{cmp, fmt};

use crate::element::ParseAs;
use crate::{Error, ErrorKind};

/// Mode defined by a line tag.
///
/// See [MXP specification: MXP Line Tags](https://www.zuggsoft.com/zmud/mxp.htm#MXP%20Line%20Tags).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mode(pub u8);

impl Mode {
    /// For this line, only MXP commands in the "open" category are allowed.
    pub const OPEN: Self = Self(0);
    /// For this line, all tags and commands in MXP are allowed.
    pub const SECURE: Self = Self(1);
    /// For this line, no MXP or HTML commands are allowed.
    /// The line is not parsed for any tags at all.
    pub const LOCKED: Self = Self(2);
    /// Close all open tags.
    pub const RESET: Self = Self(3);
    /// Next tag is secure only.
    pub const SECURE_ONCE: Self = Self(4);
    /// Until mode changes, only MXP commands in the "open" category are allowed.
    pub const PERM_OPEN: Self = Self(5);
    /// Until mode changes, all tags and commands in MXP are allowed.
    pub const PERM_SECURE: Self = Self(6);
    /// Until mode changes, no MXP or HTML commands are allowed.
    /// Lines aare not parsed for any tags at all.
    pub const PERM_LOCKED: Self = Self(7);

    /// The line is parsed as the name of a room.
    pub const ROOM_NAME: Self = Self(10);
    /// The line is parsed as the description of a room.
    pub const ROOM_DESC: Self = Self(11);
    /// The line is parsed as the name of a room.
    pub const ROOM_EXITS: Self = Self(12);
    /// This text is sent from the MUD at the beginning of a session to welcome the user to the MUD. Same as the <WELCOME> MXP tag.
    pub const WELCOME_TEXT: Self = Self(19);

    /// Minimum value of a user-defined line mode. See [`Mode::is_user_defined`].
    pub const USER_DEFINED_MIN: Self = Self(20);
    /// Maximum value of a user-defined line mode. See [`Mode::is_user_defined`].
    pub const USER_DEFINED_MAX: Self = Self(99);
}

impl Mode {
    /// When this mode is active, only MXP commands in the "open" category are allowed.
    pub const fn is_open(self) -> bool {
        matches!(self, Self::OPEN | Self::PERM_OPEN)
    }

    /// When this mode is active, all tags and commands in MXP are allowed.
    pub const fn is_secure(self) -> bool {
        matches!(self, Self::SECURE | Self::SECURE_ONCE | Self::PERM_SECURE)
    }

    /// When this mode is active, MXP tags should not be parsed.
    pub const fn is_locked(self) -> bool {
        matches!(self, Self::LOCKED | Self::PERM_LOCKED)
    }

    /// When this mode is active, the line is tagged for automappers.
    pub const fn is_automapping(self) -> bool {
        matches!(self.0, 10..=19)
    }

    /// How the line should be parsed by an automapper. This only applies to [`Mode::ROOM_NAME`],
    /// [`Mode::ROOM_DESC`], and [`Mode::ROOM_EXITS`].
    pub const fn parse_as(self) -> Option<ParseAs> {
        match self {
            Self::ROOM_NAME => Some(ParseAs::RoomName),
            Self::ROOM_DESC => Some(ParseAs::RoomDesc),
            Self::ROOM_EXITS => Some(ParseAs::RoomExit),
            _ => None,
        }
    }

    /// Returns `true` if this mode is [`Mode::WELCOME_TEXT`].
    pub const fn is_welcome(self) -> bool {
        matches!(self, Self::WELCOME_TEXT)
    }

    /// When this mode is active, the line is tagged for a MUD-specific purpose. The tag must have
    /// been previously defined using an MXP definition command to tell the client how to handle the
    /// line.
    ///
    /// A common use for these is to tag various chat channels to allow client-side filtering or
    /// redirection of the text.
    pub const fn is_user_defined(self) -> bool {
        const MIN: u8 = Mode::USER_DEFINED_MIN.0;
        const MAX: u8 = Mode::USER_DEFINED_MAX.0;
        matches!(self.0, MIN..=MAX)
    }

    pub fn user(mode: u8) -> crate::Result<Self> {
        let mode = Self(mode);
        if mode.is_user_defined() {
            Ok(mode)
        } else {
            Err(Error::new(
                mode.0.to_string(),
                ErrorKind::InvalidLineTagMode,
            ))
        }
    }
}

impl PartialEq<u8> for Mode {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}
impl PartialEq<Mode> for u8 {
    fn eq(&self, other: &Mode) -> bool {
        *self == other.0
    }
}
impl PartialOrd<u8> for Mode {
    fn partial_cmp(&self, other: &u8) -> Option<cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}
impl PartialOrd<Mode> for u8 {
    fn partial_cmp(&self, other: &Mode) -> Option<cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ModeRangeError(pub(crate) ());

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
                    Err(ModeRangeError(()))
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
                    Err(ModeRangeError(()))
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
