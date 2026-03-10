use std::{cmp, fmt};

use flagset::FlagSet;

use super::element::ParseAs;
use crate::keyword::MxpKeyword;

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

    /// When this mode is active, MXP tags should be parsed.
    pub const fn is_mxp(self) -> bool {
        matches!(
            self,
            Self::OPEN | Self::PERM_OPEN | Self::SECURE | Self::SECURE_ONCE | Self::PERM_SECURE
        )
    }

    /// When this mode is active, all tags and commands in MXP are allowed.
    pub const fn is_secure(self) -> bool {
        matches!(self, Self::SECURE | Self::SECURE_ONCE | Self::PERM_SECURE)
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

/// State tracker for
///
/// See [MXP specification: MXP Line Tags](https://www.zuggsoft.com/zmud/mxp.htm#MXP%20Line%20Tags).
#[allow(clippy::struct_field_names)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModeState {
    active_mode: Mode,
    default_mode: Mode,
    previous_mode: Mode,
}

impl Default for ModeState {
    fn default() -> Self {
        Self::new()
    }
}

impl ModeState {
    /// Constructs a new `ModeState` in [`Mode::OPEN`] mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::ModeState;
    ///
    /// let mode_state = ModeState::new();
    /// assert_eq!(mode_state, mxp::Mode::OPEN);
    /// ```
    pub const fn new() -> Self {
        Self {
            active_mode: Mode::OPEN,
            default_mode: Mode::OPEN,
            previous_mode: Mode::OPEN,
        }
    }

    /// Gets the active mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::ModeState;
    ///
    /// let mut mode_state = ModeState::new();
    /// mode_state.set(mxp::Mode::SECURE);
    /// assert_eq!(mode_state, mxp::Mode::SECURE);
    /// ```
    pub const fn get(&self) -> Mode {
        self.active_mode
    }

    /// Applies a new line mode. If the line defines a specific mode, use `self.update(Some(mode))`.
    /// Otherwise, use `self.update(None)` to restore the default mode.
    ///
    /// Returns `true` if the previous mode was open but the new mode is not, meaning all tags
    /// since the most recent unsecure tag should be closed.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::ModeState;
    ///
    /// let mut mode_state = ModeState::new();
    /// mode_state.update(Some(mxp::Mode::PERM_OPEN));
    /// mode_state.update(Some(mxp::Mode::SECURE_ONCE));
    /// assert_eq!(mode_state, mxp::Mode::SECURE_ONCE);
    /// mode_state.update(None);
    /// assert_eq!(mode_state, mxp::Mode::PERM_OPEN);
    /// ```
    pub const fn update(&mut self, mode: Option<Mode>) -> bool {
        self.set(match mode {
            Some(mode) => mode,
            None => self.default_mode,
        })
    }

    /// Alias for `self.update(Some(mode))`.
    /// See the documentation for [`update`](Self::update) for more.
    pub const fn set(&mut self, mode: Mode) -> bool {
        let closing = self.active_mode.is_open() && !mode.is_open();
        match mode {
            Mode::OPEN | Mode::SECURE | Mode::LOCKED => {
                self.default_mode = Mode::OPEN;
            }
            Mode::SECURE_ONCE => self.previous_mode = self.active_mode,
            Mode::PERM_OPEN | Mode::PERM_SECURE | Mode::PERM_LOCKED => {
                self.default_mode = mode;
            }
            _ => (),
        }
        self.active_mode = mode;
        closing
    }

    /// Alias for `self.update(None)`.
    /// See the documentation for [`update`](Self::update) for more.
    pub const fn revert(&mut self) -> bool {
        self.set(self.default_mode)
    }

    /// Applies keywords that affect the default mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::ModeState;
    ///
    /// let mut mode_state = ModeState::new();
    /// assert_eq!(mode_state, mxp::Mode::OPEN);
    /// mode_state.apply_keywords(mxp::MxpKeyword::DefaultLocked);
    /// mode_state.revert();
    /// assert_eq!(mode_state, mxp::Mode::LOCKED);
    /// mode_state.apply_keywords(mxp::MxpKeyword::DefaultSecure);
    /// mode_state.revert();
    /// assert_eq!(mode_state, mxp::Mode::SECURE);
    /// mode_state.apply_keywords(mxp::MxpKeyword::DefaultOpen);
    /// mode_state.revert();
    /// assert_eq!(mode_state, mxp::Mode::OPEN);
    /// ```
    pub fn apply_keywords<T: Into<FlagSet<MxpKeyword>>>(&mut self, keywords: T) {
        let keywords = keywords.into();
        if keywords.contains(MxpKeyword::DefaultLocked) {
            self.default_mode = Mode::LOCKED;
        } else if keywords.contains(MxpKeyword::DefaultSecure) {
            self.default_mode = Mode::SECURE;
        } else if keywords.contains(MxpKeyword::DefaultOpen) {
            self.default_mode = Mode::OPEN;
        }
    }

    /// Returns `true` if the active mode is secure. Additionally, if the active mode is
    /// [`Mode::SECURE_ONCE`], the active mode reverts to the previous mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::ModeState;
    ///
    /// let mut mode_state = ModeState::new();
    /// mode_state.set(mxp::Mode::PERM_OPEN);
    /// mode_state.revert();
    /// assert_eq!(mode_state, mxp::Mode::PERM_OPEN);
    /// mode_state.set(mxp::Mode::SECURE_ONCE);
    /// assert!(mode_state.use_secure());
    /// assert_eq!(mode_state, mxp::Mode::PERM_OPEN);
    /// mode_state.set(mxp::Mode::SECURE);
    /// assert!(mode_state.use_secure());
    /// assert_eq!(mode_state, mxp::Mode::SECURE);
    /// ```
    pub const fn use_secure(&mut self) -> bool {
        if matches!(self.active_mode, Mode::SECURE_ONCE) {
            self.active_mode = self.previous_mode;
            true
        } else {
            self.active_mode.is_secure()
        }
    }

    /// See [`Mode::is_open`].
    pub const fn is_open(&self) -> bool {
        self.get().is_open()
    }

    /// See [`Mode::is_mxp`].
    pub const fn is_mxp(&self) -> bool {
        self.get().is_mxp()
    }

    /// See [`Mode::is_secure`].
    pub const fn is_secure(&self) -> bool {
        self.get().is_secure()
    }

    /// See [`Mode::is_automapping`].
    pub const fn is_automapping(&self) -> bool {
        self.get().is_secure()
    }

    /// See [`Mode::parse_as`].
    pub fn parse_as(&self) -> Option<ParseAs> {
        self.get().parse_as()
    }

    /// See [`Mode::is_welcome`].
    pub const fn is_welcome(&self) -> bool {
        self.get().is_welcome()
    }

    /// See [`Mode::is_mxp`].
    pub const fn is_user_defined(&self) -> bool {
        self.get().is_user_defined()
    }
}

impl PartialEq<Mode> for ModeState {
    fn eq(&self, other: &Mode) -> bool {
        self.active_mode == *other
    }
}
impl PartialEq<ModeState> for Mode {
    fn eq(&self, other: &ModeState) -> bool {
        *self == other.active_mode
    }
}
impl PartialOrd<Mode> for ModeState {
    fn partial_cmp(&self, other: &Mode) -> Option<cmp::Ordering> {
        self.active_mode.partial_cmp(other)
    }
}
impl PartialOrd<ModeState> for Mode {
    fn partial_cmp(&self, other: &ModeState) -> Option<cmp::Ordering> {
        self.partial_cmp(&other.active_mode)
    }
}
impl PartialEq<u8> for ModeState {
    fn eq(&self, other: &u8) -> bool {
        self.active_mode.0 == *other
    }
}
impl PartialEq<ModeState> for u8 {
    fn eq(&self, other: &ModeState) -> bool {
        *self == other.active_mode.0
    }
}
impl PartialOrd<u8> for ModeState {
    fn partial_cmp(&self, other: &u8) -> Option<cmp::Ordering> {
        self.active_mode.0.partial_cmp(other)
    }
}
impl PartialOrd<ModeState> for u8 {
    fn partial_cmp(&self, other: &ModeState) -> Option<cmp::Ordering> {
        self.partial_cmp(&other.active_mode.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ModeRangeError(());

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
