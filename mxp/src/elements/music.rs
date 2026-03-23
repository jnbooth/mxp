use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use super::AudioRepetition;
use crate::arguments::{ArgumentScanner, Arguments, ExpectArg as _};
use crate::parse::{Decoder, StringVariant, UnrecognizedVariant};

#[derive(Copy, Clone, Default, PartialEq, Eq)]
enum AudioContinuation {
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

impl fmt::Display for AudioContinuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Restart => 0,
            Self::Continue => 1,
        }
        .fmt(f)
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
/// assert_eq!(
///     "<MUSIC 'berlioz/fantas?' V=80 L=3 C=1 T=music U='http://example.org:5000/music'>".parse::<mxp::Music>(),
///     Ok(mxp::Music {
///         fname: "berlioz/fantas?".into(),
///         volume: 80,
///         repeat: 3.try_into().unwrap(),
///         continual: true,
///         class: Some("music".into()),
///         url: Some("http://example.org:5000/music".into()),
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Music<S = String> {
    /// File name. May contain wildcards. If no extension is specified, ".midi" should be assumed.
    pub fname: S,
    /// Volume between 0 and 100.
    pub volume: u8,
    /// Repeat behavior.
    pub repeat: AudioRepetition,
    /// If requested again, the file should simply continue playing..
    pub continual: bool,
    /// Type of music, e.g. combat, zone, death, clan. Case-insensitive. This parameter was
    /// intended to provide a way to group music into subfolders within the main music directory.
    pub class: Option<S>,
    /// Specifies the URL of the music file. This allows downloading files from the MUD server.
    /// Client should always look in local directories first, and only download the file if it's
    /// not available locally.
    pub url: Option<S>,
}

impl<S: Default> Default for Music<S> {
    fn default() -> Self {
        Self {
            fname: S::default(),
            volume: 100,
            repeat: AudioRepetition::default(),
            continual: false,
            class: None,
            url: None,
        }
    }
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

impl<'a> Music<&'a str> {
    /// Parses a !!MUSIC element from an MSP string.
    ///
    /// # Examples
    ///
    /// ```
    /// let msp_string = "!!MUSIC(fugue.mid V=80 L=3 C=1 T=music U=http://example.org:5000/music)";
    /// let msp_trimmed = &msp_string[8..msp_string.len() - 1];
    /// assert_eq!(
    ///     mxp::Music::from_msp(msp_trimmed),
    ///     Ok(mxp::Music {
    ///         fname: "fugue.mid",
    ///         volume: 80,
    ///         repeat: 3.try_into().unwrap(),
    ///         continual: true,
    ///         class: Some("music"),
    ///         url: Some("http://example.org:5000/music"),
    ///     }),
    /// );
    /// ```
    pub fn from_msp(source: &'a str) -> crate::Result<Self> {
        Self::scan(Arguments::parse(source)?.into_scan())
    }
}

impl<'a, S: AsRef<str>> Music<S> {
    pub(crate) fn scan<A>(mut scanner: A) -> crate::Result<Self>
    where
        A: ArgumentScanner<'a, Decoded = S>,
    {
        let fname = scanner.get_next_or("fname")?.expect_some("fname")?;
        let volume = scanner.get_next_or("v")?.expect_number()?.unwrap_or(100);
        let repeat = scanner
            .get_next_or("l")?
            .expect_number()?
            .unwrap_or_default();
        let continual =
            scanner.get_next_or("c")?.expect_variant()? == Some(AudioContinuation::Continue);
        let class = scanner.get_next_or("t")?;
        let url = scanner.get_next_or("u")?;
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

impl_from_str!(Music);

impl<S: AsRef<str>> fmt::Display for Music<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Music {
            fname,
            volume,
            repeat,
            continual,
            class,
            url,
        } = self.borrow_text();
        crate::display::ElementFormatter {
            name: "MUSIC",
            arguments: &[
                &fname,
                &(volume, 100),
                &(repeat, AudioRepetition::default()),
                &(u8::from(continual), 0),
                &class,
                &url,
            ],
            keywords: &[],
        }
        .fmt(f)
    }
}
