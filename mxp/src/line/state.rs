use super::mode::Mode;
use super::tag::LineTag;
use crate::element::ParseAs;
use crate::{Error, ErrorKind};

/// State tracker for [`Mode`].
///
/// See [MXP specification: MXP Line Tags](https://www.zuggsoft.com/zmud/mxp.htm#User-defined%20Line%20Tags).
//
// Note: these modes are never PERM_LOCKED, PERM_OPEN, PERM_SECURE, or RESET.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModeState {
    active_mode: Mode,
    default_mode: Mode,
    previous_mode: Mode,
}

impl Default for ModeState {
    /// See [`ModeState::new`].
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
    /// assert_eq!(mode_state.get(), mxp::Mode::OPEN);
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
    /// assert_eq!(mode_state.get(), mxp::Mode::SECURE);
    /// ```
    #[inline]
    pub const fn get(&self) -> Mode {
        self.active_mode
    }

    /// Retrieves the element associated with a line tag for a specified mode, if one exists.
    pub fn line_tag<'a>(&self, state: &'a crate::State) -> Option<LineTag<'a>> {
        state.get_line_tag(self.active_mode)
    }

    const fn lock(&mut self, mode: Mode) {
        self.active_mode = mode;
        self.default_mode = mode;
    }

    /// Applies a new line mode. Returns `true` if the change in mode means all tags since the
    /// most recent OPEN tag should be closed.
    ///
    /// Note: If the mode is set to [`Mode::SECURE_ONCE`], the client must immediately process an
    /// incoming element and call [`ModeState::use_secure`] for it.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::ModeState;
    ///
    /// let mut mode_state = ModeState::new();
    /// mode_state.set(mxp::Mode::SECURE);
    /// assert_eq!(mode_state.get(), mxp::Mode::SECURE);
    /// mode_state.set(mxp::Mode::OPEN);
    /// assert_eq!(mode_state.get(), mxp::Mode::OPEN);
    /// ```
    pub const fn set(&mut self, mode: Mode) -> bool {
        let closing = self.is_open() && !mode.is_open();
        match mode {
            Mode::SECURE_ONCE => {
                self.previous_mode = self.active_mode;
                self.active_mode = mode;
            }
            Mode::PERM_OPEN | Mode::RESET => self.lock(Mode::OPEN),
            Mode::PERM_SECURE => self.lock(Mode::SECURE),
            Mode::PERM_LOCKED => self.lock(Mode::LOCKED),
            _ => self.active_mode = mode,
        }
        closing
    }

    /// Revert to the default line mode. This function should be called whenever the client receives
    /// a newline.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::ModeState;
    ///
    /// let mut mode_state = ModeState::new();
    /// mode_state.set(mxp::Mode::PERM_LOCKED);
    /// mode_state.set(mxp::Mode::SECURE);
    /// mode_state.revert();
    /// assert_eq!(mode_state.get(), mxp::Mode::LOCKED);
    /// ```
    pub const fn revert(&mut self) {
        self.set(self.default_mode);
    }

    /// Returns `true` if the active mode is SECURE. Additionally, if the active mode is
    /// [`Mode::SECURE_ONCE`], the active mode reverts to the previous mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::ModeState;
    ///
    /// let mut mode_state = ModeState::new();
    /// mode_state.set(mxp::Mode::OPEN);
    /// assert!(!mode_state.use_secure());
    ///
    /// mode_state.set(mxp::Mode::SECURE);
    /// assert!(mode_state.use_secure());
    /// assert!(mode_state.use_secure()); // still secure
    ///
    /// mode_state.set(mxp::Mode::OPEN);
    /// mode_state.set(mxp::Mode::SECURE_ONCE);
    /// assert!(mode_state.use_secure());
    /// assert!(!mode_state.use_secure()); // no longer secure
    /// ```
    #[inline]
    pub const fn use_secure(&mut self) -> bool {
        if self.is_secure_once() {
            self.active_mode = self.previous_mode;
            return true;
        }
        !self.is_open()
    }

    /// If the active mode is [`Mode::SECURE_ONCE`] and the character byte is not `b'<'`, unsets
    /// the mode and returns an error.
    #[inline]
    pub fn validate_next_character(&mut self, c: u8) -> crate::Result<()> {
        #[cold]
        #[inline(never)]
        fn create_error(c: u8) -> Error {
            Error::new(c as char, ErrorKind::TextAfterSecureOnce)
        }

        if c == b'<' || !self.is_secure_once() {
            return Ok(());
        }
        self.active_mode = self.previous_mode;
        Err(create_error(c))
    }

    /// Returns `true` if the active mode is [`Mode::SECURE_ONCE`].
    ///
    /// If this function returns `true`, the server must send `'<'` as its next character to start a
    /// tag.
    #[inline]
    pub const fn is_secure_once(&self) -> bool {
        self.active_mode.0 == Mode::SECURE_ONCE.0
    }

    /// See [`Mode::is_open`].
    #[inline]
    pub const fn is_open(&self) -> bool {
        self.active_mode.0 == Mode::OPEN.0 || self.active_mode.0 >= Mode::USER_DEFINED_MIN.0
    }

    /// See [`Mode::is_locked`].
    #[inline]
    pub const fn is_locked(&self) -> bool {
        self.active_mode.0 == Mode::LOCKED.0
    }

    /// See [`Mode::is_automapping`].
    #[inline]
    pub const fn is_automapping(&self) -> bool {
        self.active_mode.is_automapping()
    }

    /// See [`Mode::parse_as`].
    #[inline]
    pub fn parse_as(&self) -> Option<ParseAs> {
        self.active_mode.parse_as()
    }

    /// See [`Mode::is_user_defined`].
    #[inline]
    pub const fn is_user_defined(&self) -> bool {
        self.active_mode.is_user_defined()
    }
}
