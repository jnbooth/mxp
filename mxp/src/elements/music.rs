use std::borrow::Cow;
use std::str::FromStr;

use super::AudioRepetition;
use crate::arguments::ExpectArg as _;
use crate::parse::{Decoder, Scan, StringVariant, UnrecognizedVariant};

/// Specifies file behavior if the server requests it should play again while it is already playing.
///
/// See [MSP specification: C parameter](https://www.zuggsoft.com/zmud/msp.htm#MSP%20Specification).
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum AudioContinuation {
    /// If requested again, the file should restart.
    Restart = 0,
    #[default]
    /// If requested again, the file should simply continue playing.
    Continue,
}

impl StringVariant for AudioContinuation {
    const VARIANTS: &[&str] = &["0", "1"];
}

impl FromStr for AudioContinuation {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::Restart),
            "1" => Ok(Self::Continue),
            _ => Err(Self::Err::new(s)),
        }
    }
}

/// Music triggers are typically background MID (MIDI) files. Only one music trigger can be active
/// at once.
///
/// See [MXP specification: `<MUSIC>`](https://www.zuggsoft.com/zmud/mxp.htm#MSP%20Compatibility)
/// and the [MSP (Mud Sound Protocol) specification](https://www.zuggsoft.com/zmud/msp.htm).
///
/// # Examples
///
/// ```
/// use mxp::AudioRepetition;
///
/// assert_eq!(
///     "<MUSIC 'berlioz/fantas?' V=80 L=3 C=1 T=music U='http://example.org:5000/music'>".parse::<mxp::Music>(),
///     Ok(mxp::Music {
///         fname: "berlioz/fantas?".into(),
///         volume: 80,
///         repeat: AudioRepetition::Count(3.try_into().unwrap()),
///         continual: true,
///         class: Some("music".into()),
///         url: Some("http://example.org:5000/music".into()),
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Music<S = String> {
    /// File name. May contain wildcards. If no extension is specified, ".midi" should be assumed.
    pub fname: S,
    /// Volume between 0 and 100.
    pub volume: u8,
    /// Repeat behavior.
    pub repeat: AudioRepetition,
    /// If requested again, the file should simply continue playing.
    pub continual: bool,
    /// Type of music, e.g. combat, zone, death, clan. Case-insensitive. This parameter was
    /// intended to provide a way to group music into subfolders within the main music directory.
    pub class: Option<S>,
    /// Specifies the URL of the music file. This allows downloading files from the MUD server.
    /// Client should always look in local directories first, and only download the file if it's
    /// not available locally.
    pub url: Option<S>,
}

impl<S: AsRef<str>> Music<S> {
    /// Returns `true` if this command is a `<MUSIC OFF>` command, causing music to stop rather
    /// than triggering music.
    pub fn is_off(&self) -> bool {
        self.fname.as_ref().eq_ignore_ascii_case("off") && self.url.is_none()
    }
}

impl<S> Music<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, mut f: F) -> Music<T>
    where
        F: FnMut(S) -> T,
    {
        Music {
            fname: f(self.fname),
            volume: self.volume,
            repeat: self.repeat,
            continual: self.continual,
            class: self.class.map(&mut f),
            url: self.url.map(f),
        }
    }
}

impl_into_owned!(Music);

impl<S: AsRef<str>> Music<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Music<&str> {
        Music {
            fname: self.fname.as_ref(),
            volume: self.volume,
            repeat: self.repeat,
            continual: self.continual,
            class: self.class.as_ref().map(AsRef::as_ref),
            url: self.url.as_ref().map(AsRef::as_ref),
        }
    }
}

impl_partial_eq!(Music);

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Music<Cow<'a, str>> {
    type Error = crate::Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let fname = scanner.next_or("fname")?.expect_some("fname")?;
        let volume = scanner.next_or("v")?.expect_number()?.unwrap_or(100);
        let repeat = scanner.next_or("l")?.expect_number()?.unwrap_or_default();
        let continual =
            scanner.next_or("c")?.expect_variant()? == Some(AudioContinuation::Continue);
        let class = scanner.next_or("t")?;
        let url = scanner.next_or("u")?;
        scanner.expect_end()?;
        Ok(Self {
            fname,
            volume,
            repeat,
            continual,
            class,
            url,
        })
    }
}

impl FromStr for Music {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Music)
    }
}
