use std::{cmp, fmt};

use crate::element::ParseAs;

/// Mode defined by a line tag.
///
/// See [MXP specification: MXP Line Tags](https://www.zuggsoft.com/zmud/mxp.htm#User-defined%20Line%20Tags).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mode(pub u8);

impl Mode {
    /// Only MXP commands in the OPEN category are allowed.
    /// When a newline is received from the MUD, the mode reverts back to the default mode.
    /// This mode starts as the default mode.
    pub const OPEN: Self = Self(0);
    /// All tags and commands in MXP are allowed within the line.
    /// When a newline is received from the MUD, the mode reverts back to the default mode.
    pub const SECURE: Self = Self(1);
    /// No MXP or HTML commands are allowed in the line. The line is not parsed for any tags at all.
    /// When a newline is received from the MUD, the mode reverts back to the Default mode.
    pub const LOCKED: Self = Self(2);
    /// Close all open tags.  Set active mode and default mode to [`OPEN`](Self::OPEN).
    /// Set text color and properties to default.
    pub const RESET: Self = Self(3);
    /// Set [`SECURE`](Self::SECURE) mode for the next tag only.
    /// Must be immediately followed by a `'<'` character to start a tag.
    pub const SECURE_ONCE: Self = Self(4);
    /// Set [`OPEN`](Self::OPEN) mode. Mode remains in effect until changed.
    /// [`OPEN`](Self::OPEN) mode becomes the new default mode.
    pub const PERM_OPEN: Self = Self(5);
    /// Set [`SECURE`](Self::SECURE) mode. Mode remains in effect until changed.
    /// [`SECURE`](Self::SECURE) mode becomes the new default mode.
    pub const PERM_SECURE: Self = Self(6);
    /// Set [`LOCKED`](Self::LOCKED) mode. Mode remains in effect until changed.
    /// [`LOCKED`](Self::LOCKED) mode becomes the new default mode.
    pub const PERM_LOCKED: Self = Self(7);

    /// The line is parsed as the name of a room.
    pub const ROOM_NAME: Self = Self(10);
    /// The line is parsed as the description of a room.
    pub const ROOM_DESC: Self = Self(11);
    /// The line is parsed as the name of a room.
    pub const ROOM_EXITS: Self = Self(12);
    /// This text is sent from the MUD at the beginning of a session to welcome the user to the MUD.
    pub const WELCOME_TEXT: Self = Self(19);

    /// Minimum value of a user-defined line mode. See [`Mode::is_user_defined`].
    pub const USER_DEFINED_MIN: Self = Self(20);
    /// Maximum value of a user-defined line mode. See [`Mode::is_user_defined`].
    pub const USER_DEFINED_MAX: Self = Self(99);
}

impl Mode {
    /// When this mode is active, only MXP commands in the OPEN category are allowed.
    ///
    /// The following modes are OPEN:
    ///
    /// - [`OPEN`](Self::OPEN)
    /// - [`PERM_OPEN`](Self::PERM_OPEN)
    /// - User defined line tags (see [`is_user_defined`](Self::is_user_defined))
    #[inline]
    pub const fn is_open(self) -> bool {
        const OPEN: u8 = Mode::OPEN.0;
        const PERM_OPEN: u8 = Mode::PERM_OPEN.0;
        const MIN_USER_DEFINED: u8 = Mode::USER_DEFINED_MIN.0;

        matches!(self.0, OPEN | PERM_OPEN | MIN_USER_DEFINED..)
    }

    /// When this mode is active, MXP tags should not be parsed.
    ///
    /// The following modes are locked:
    ///
    /// - [`LOCKED`](Self::LOCKED)
    /// - [`PERM_LOCKED`](Self::PERM_LOCKED)
    #[inline]
    pub const fn is_locked(self) -> bool {
        matches!(self, Self::LOCKED | Self::PERM_LOCKED)
    }

    /// When this mode is active, the line is tagged for automappers.
    ///
    /// Line modes between 10 [`ROOM_NAME`](Self::ROOM_NAME) and 19
    /// [`WELCOME_TEXT`](Self::WELCOME_TEXT) are reserved for automappers.
    #[inline]
    pub const fn is_automapping(self) -> bool {
        matches!(self.0, 10..=19)
    }

    /// How the line should be parsed by an automapper. This only applies to
    /// [`ROOM_NAME`](Self::ROOM_NAME), [`Mode::ROOM_DESC`](Self::ROOM_DESC), and
    /// [`ROOM_EXITS`](Self::ROOM_EXITS).
    #[inline]
    pub const fn parse_as(self) -> Option<ParseAs> {
        match self {
            Self::ROOM_NAME => Some(ParseAs::RoomName),
            Self::ROOM_DESC => Some(ParseAs::RoomDesc),
            Self::ROOM_EXITS => Some(ParseAs::RoomExit),
            _ => None,
        }
    }

    /// When this mode is active, the line is tagged for a MUD-specific purpose. The tag must have
    /// been previously defined using an MXP definition command to tell the client how to handle the
    /// line.
    ///
    /// A common use for these is to tag various chat channels to allow client-side filtering or
    /// redirection of the text.
    ///
    /// User-defined tags must be between 20 ([`USER_DEFINED_MIN`](Self::USER_DEFINED_MIN)) and
    /// 99 ([`USER_DEFINED_MAX`](Self::USER_DEFINED_MAX)).
    #[inline]
    pub const fn is_user_defined(self) -> bool {
        const MIN: u8 = Mode::USER_DEFINED_MIN.0;
        const MAX: u8 = Mode::USER_DEFINED_MAX.0;

        matches!(self.0, MIN..=MAX)
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
impl PartialEq<usize> for Mode {
    fn eq(&self, other: &usize) -> bool {
        usize::from(self.0) == *other
    }
}
impl PartialEq<Mode> for usize {
    fn eq(&self, other: &Mode) -> bool {
        *self == usize::from(other.0)
    }
}
impl PartialOrd<usize> for Mode {
    fn partial_cmp(&self, other: &usize) -> Option<cmp::Ordering> {
        usize::from(self.0).partial_cmp(other)
    }
}
impl PartialOrd<Mode> for usize {
    fn partial_cmp(&self, other: &Mode) -> Option<cmp::Ordering> {
        self.partial_cmp(&usize::from(other.0))
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Mode> for usize {
    fn from(value: Mode) -> Self {
        value.0.into()
    }
}

/// The error type returned when a checked conversion from an integral type to [`Mode`] fails.
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
