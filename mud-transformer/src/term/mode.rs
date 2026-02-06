use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    Standard(u16),
    Private(u16),
}

impl Mode {
    /// KAM (Keyboard Action Mode)
    pub const KEYBOARD_ACTION: Self = Self::Standard(2);
    /// CRM (Show Control Character Mode)
    pub const SHOW_CONTROL_CHARACTER: Self = Self::Standard(3);
    /// IRM (Insert/Replace Mode)
    pub const INSERT: Self = Self::Standard(4);
    /// SRM (Local Echo: Send/Receive Mode)
    pub const NO_ECHO: Self = Self::Standard(12);
    /// LNM (Line Feed/New Line Mode)
    pub const NEW_LINE: Self = Self::Standard(20);

    /// DECCKM (Cursor Keys Mode)
    pub const CURSOR_KEYS: Self = Self::Private(1);
    /// DECANM (ANSI Mode)
    pub const ANSI: Self = Self::Private(2);
    /// DECCOLM (Select 80 or 132 Columns per Page)
    pub const WIDE: Self = Self::Private(3);
    /// DECSCLM (Scrolling Mode)
    pub const SCROLLING_MODE: Self = Self::Private(4);
    /// DECSCNM (Screen Mode: Light or Dark Screen)
    pub const LIGHT_BACKGROUND: Self = Self::Private(5);
    /// DECOM (Origin Mode)
    pub const ORIGIN: Self = Self::Private(6);
    /// DECAWM (Autowrap Mode)
    pub const AUTOWRAP: Self = Self::Private(7);
    /// DECARM (Autorepeat Mode)
    pub const AUTOREPEAT: Self = Self::Private(8);
    /// Send Mouse X & Y on button press
    pub const TRACK_PRESS: Self = Self::Private(9);
    /// Show toolbar (rxvt)
    pub const TOOLBAR: Self = Self::Private(10);
    /// Start Blinking Cursor (att610)
    pub const BLINK_CURSOR: Self = Self::Private(12);
    /// DECPFF (Print Form Feed Mode)
    pub const PRINT_FORM_FEED: Self = Self::Private(18);
    /// DECPEX (Printer Extend Mode)
    pub const PRINT_FULL_PAGE: Self = Self::Private(19);
    /// DECTCEM (Text Cursor Enable Mode)
    pub const TEXT_CURSOR: Self = Self::Private(25);
    /// Show scrollbar (rxvt)
    pub const SCROLLBAR: Self = Self::Private(30);
    /// DECRLM (Cursor Right to Left Mode)
    pub const RIGHT_TO_LEFT: Self = Self::Private(34);
    /// Enable font-shifting functions (rxvt)
    pub const FONT_SHIFTING: Self = Self::Private(35);
    /// DECHEBM (Hebrew/N-A Keyboard Mapping)
    pub const HEBREW_MAPPING: Self = Self::Private(35);
    /// DECHEM (Hebrew Encoding Mode)
    pub const HEBREW_ENCODING: Self = Self::Private(36);
    /// Enter Tektronix Mode (DECTEK)
    pub const TEKTRONIX: Self = Self::Private(38);
    /// Allow 80 → 132 Mode
    pub const ALLOW_WIDE: Self = Self::Private(40);
    /// more(1) fix
    pub const MORE_1_FIX: Self = Self::Private(41);
    /// DECNRCM (National Replacement Character Set Mode)
    pub const CHARSET_8_BIT: Self = Self::Private(42);
    /// Turn on margin bell
    pub const MARGIN_BELL: Self = Self::Private(44);
    /// Reverse-wraparound mode
    pub const REVERSE_WRAP: Self = Self::Private(45);
    /// Start logging
    pub const LOGGING: Self = Self::Private(46);
    /// Use Alternate Screen Buffer
    pub const ALTERNATE_SCREEN_BUFFER: Self = Self::Private(47);
    /// DECNAKB (Greek/N-A Keyboard Mapping)
    pub const GREEK_MAPPING: Self = Self::Private(57);
    /// DECIPEM (IBM ProPrinter Emulation Mode)
    pub const PRO_PRINTER: Self = Self::Private(58);
    /// DECVCCM (Vertical Cursor-Coupling Mode)
    pub const VERTICAL_CURSOR_COUPLING: Self = Self::Private(61);
    /// DECPCCM (Page Cursor-Coupling Mode)
    pub const PAGE_CURSOR_COUPLING: Self = Self::Private(64);
    /// DECNKM (Numeric Keypad Mode)
    pub const NUMERIC_KEYPAD: Self = Self::Private(66);
    /// DECBKM (Backarrow Key Mode)
    pub const BACKARROW_KEY: Self = Self::Private(67);
    /// DECLRMM (Left Right Margin Mode)
    pub const HMARGIN: Self = Self::Private(69);
    /// DECKBUM (Typewriter or Data Processing Keys)
    pub const DATA_PROCESSING_KEYS: Self = Self::Private(68);
    /// DECKPM (Key Position Mode)
    pub const KEY_POSITION: Self = Self::Private(81);
    /// DECNCSM (No Clearing Screen On Column Change)
    pub const KEEP_SCREEN_ON_RESIZE: Self = Self::Private(95);
    /// DECRLCM (Right-to-Left Copy)
    pub const RIGHT_TO_LEFT_COPY: Self = Self::Private(96);
    /// DECCRTSM (CRT Save Mode)
    pub const CRT_SAVER: Self = Self::Private(97);
    /// DECMCM (Modem Control Mode)
    pub const MODEM_CONTROL: Self = Self::Private(99);
    /// DECAAM (Auto Answerback Mode)
    pub const AUTO_ANSWERBACK: Self = Self::Private(100);
    /// DECCANSM (Conceal Answerback Message)
    pub const CONCEAL_ANSWERBACK: Self = Self::Private(101);
    /// DECNULM (Ignoring Null Mode)
    pub const IGNORE_NULL: Self = Self::Private(102);
    /// DECHDPXM (Half-Duplex Mode)
    pub const HALF_DUPLEX: Self = Self::Private(103);
    /// DECESKM (Secondary Keyboard Language Mode)
    pub const SECONDARY_LANGUAGE: Self = Self::Private(104);
    /// DECOSCNM (Overscan Mode)
    pub const OVERSCAN: Self = Self::Private(106);
    /// DECNUMLK (Num Lock Mode)
    pub const NUM_LOCK: Self = Self::Private(108);
    /// DECCAPSLK (Caps Lock Mode)
    pub const CAPS_LOCK: Self = Self::Private(109);
    /// DECKLHIM (Keyboard LED's Host Indicator Mode)
    pub const KEYBOARD_LED_HOST_INDICATOR: Self = Self::Private(110);
    /// Send Mouse X & Y on button press and release
    pub const TRACK_PRESS_AND_RELEASE: Self = Self::Private(1000);
    /// Use Hilite Mouse Tracking
    pub const TRACK_HILITE: Self = Self::Private(1001);
    /// Use Cell Motion Mouse Tracking
    pub const TRACK_CELL_MOTION: Self = Self::Private(1002);
    /// Use All Motion Mouse Tracking
    pub const TRACK_ALL_MOTION: Self = Self::Private(1003);
    /// Scroll to bottom on tty output (rxvt)
    pub const SCROLL_ON_TTY: Self = Self::Private(1010);
    /// Scroll to bottom on key press (rxvt)
    pub const SCROLL_ON_KEY: Self = Self::Private(1011);
    /// Enable special modifiers for Alt and NumLock
    pub const SPECIAL_MODIFIERS: Self = Self::Private(1035);
    /// Send ESC when Meta modifies a key
    pub const ESC_META: Self = Self::Private(1036);
    /// Send DEL from the editing-keypad Delete key
    pub const KEYPAD_DEL: Self = Self::Private(1037);
    /// Use Alternate Screen Buffer (for terminfo)
    pub const ALTERNATE_SCREEN_BUFFER_: Self = Self::Private(1047);
    /// Save cursor as in DECSC
    pub const DECSC: Self = Self::Private(1048);
    /// Save cursor as in DECSC and use Alternate Screen Buffer, clearing it first
    pub const DECSC_AND_ALTERNATE_SCREEN_BUFFER: Self = Self::Private(1049);
    /// Sun function-key mode
    pub const FN_SUN: Self = Self::Private(1051);
    /// HP function-key mode
    pub const FN_HP: Self = Self::Private(1052);
    /// SCO function-key mode
    pub const FN_SCO: Self = Self::Private(1053);
    /// Legacy keyboard emulation (X11R6)
    pub const KBD_LEGACY: Self = Self::Private(1060);
    /// Sun/PC keyboard emulation of VT220 keyboard
    pub const KBD_VT220: Self = Self::Private(1061);
    /// Bracketed paste mode
    pub const BRACKETED_PASTE: Self = Self::Private(2004);

    pub const fn new(code: u16, private: bool) -> Self {
        if private {
            Self::Private(code)
        } else {
            Self::Standard(code)
        }
    }

    pub const fn code(self) -> u16 {
        match self {
            Self::Standard(code) | Self::Private(code) => code,
        }
    }

    pub const fn private(&self) -> bool {
        matches!(self, Self::Private(_))
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Standard(code) => write!(f, "{code}"),
            Self::Private(code) => write!(f, "?{code}"),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Mode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            return self.to_string().serialize(serializer);
        }
        match *self {
            Self::Standard(code) => u32::from(code),
            Self::Private(code) => u32::from(code) | 0x10000,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Mode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de;

        struct ModeVisitor;

        impl de::Visitor<'_> for ModeVisitor {
            type Value = Mode;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an unsigned integer in range, optionally preceded by '?'")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let (private, code) = match v.strip_prefix("?") {
                    Some(code) => (true, code),
                    _ => (false, v),
                };
                let Ok(code) = code.parse() else {
                    return Err(E::invalid_value(de::Unexpected::Str(code), &self));
                };
                Ok(Mode::new(code, private))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                let Ok(code) = (v & !0x10000).try_into() else {
                    return Err(E::invalid_value(de::Unexpected::Unsigned(v), &self));
                };
                Ok(Mode::new(code, v >= 0x10000))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                let Ok(code) = (v & !0x10000).try_into() else {
                    return Err(E::invalid_value(de::Unexpected::Signed(v), &self));
                };
                Ok(Mode::new(code, v >= 0x10000))
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(ModeVisitor)
        } else {
            deserializer.deserialize_u32(ModeVisitor)
        }
    }
}
