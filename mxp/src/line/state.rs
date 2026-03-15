use std::cmp;

use super::mode::Mode;
use crate::element::ParseAs;

/// State tracker for [`mxp::Mode`](crate::Mode).
///
/// See [MXP specification: MXP Line Tags](https://www.zuggsoft.com/zmud/mxp.htm#MXP%20Line%20Tags).
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
            Mode::RESET => {
                self.default_mode = Mode::OPEN;
                self.active_mode = Mode::OPEN;
                return closing;
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

    /// See [`Mode::is_secure`].
    pub const fn is_secure(&self) -> bool {
        self.get().is_secure()
    }

    /// See [`Mode::is_locked`].
    pub const fn is_locked(&self) -> bool {
        self.get().is_locked()
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
