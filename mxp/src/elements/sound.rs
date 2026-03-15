use std::borrow::Cow;
use std::fmt;
use std::num::{NonZero, ParseIntError};
use std::str::FromStr;

use crate::parse::{Decoder, ExpectArg as _, Scan};

/// Specifies the number of times a sound/music file should be played.
///
/// See [MSP specification: L parameter](https://www.zuggsoft.com/zmud/msp.htm#MSP%20Specification).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AudioRepetition {
    /// The file should be played infinitely, until instructed otherwise.
    Forever,
    /// The file should play this many times.
    Count(NonZero<u32>),
}

impl Default for AudioRepetition {
    /// A single play, i.e. `AudioRepetition::Count(1)`.
    fn default() -> Self {
        Self::Count(NonZero::new(1).unwrap())
    }
}

impl fmt::Display for AudioRepetition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Forever => (-1).fmt(f),
            Self::Count(amount) => amount.fmt(f),
        }
    }
}

impl FromStr for AudioRepetition {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-1" {
            return Ok(Self::Forever);
        }
        s.parse().map(Self::Count)
    }
}

/// Sound triggers are WAV format files intended for sound effects.
///
/// See [MXP specification: `<SOUND>`](https://www.zuggsoft.com/zmud/mxp.htm#MSP%20Compatibility)
/// and the [MSP (Mud Sound Protocol) specification](https://www.zuggsoft.com/zmud/msp.htm).
///
/// # Examples
///
/// ```
/// use mxp::AudioRepetition;
///
/// assert_eq!(
///     "<SOUND 'weather/rain.wav' V=80 L=3 P=10 T=combat U='http://example.org:5000/sound'>".parse::<mxp::Sound>(),
///     Ok(mxp::Sound {
///         fname: "weather/rain.wav".into(),
///         volume: 80,
///         repeat: AudioRepetition::Count(3.try_into().unwrap()),
///         priority: 10,
///         class: Some("combat".into()),
///         url: Some("http://example.org:5000/sound".into()),
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Sound<S = String> {
    /// File name. May contain wildcards. If no extension is specified, ".wav" should be assumed.
    pub fname: S,
    /// Volume between 0 and 100.
    pub volume: u8,
    /// Repeat behavior.
    pub repeat: AudioRepetition,
    /// This parameter applies when some sound is playing and another request arrives. Then, if new
    /// request has higher (but NOT equal) priority than the one that's currently being played, old
    /// sound must be stopped and the new sound starts playing instead. In the case of a tie, the
    /// sound that is already playing wins.
    pub priority: u8,
    /// Type of sound, e.g. combat, zone, death, clan. Case-insensitive. This parameter was
    /// intended to provide a way to group sounds into subfolders within the main sound directory.
    pub class: Option<S>,
    /// Specifies the URL of the sound file. This allows downloading files from the MUD server.
    /// Client should always look in local directories first, and only download the file if it's
    /// not available locally.
    pub url: Option<S>,
}

impl<S: AsRef<str>> Sound<S> {
    /// Returns `true` if this command is a `<SOUND OFF>` command, causing sounds to stop rather
    /// than triggering a sound.
    pub fn is_off(&self) -> bool {
        self.fname.as_ref().eq_ignore_ascii_case("off") && self.url.is_none()
    }
}

impl<S> Sound<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, mut f: F) -> Sound<T>
    where
        F: FnMut(S) -> T,
    {
        Sound {
            fname: f(self.fname),
            volume: self.volume,
            repeat: self.repeat,
            priority: self.priority,
            class: self.class.map(&mut f),
            url: self.url.map(f),
        }
    }
}

impl_into_owned!(Sound);

impl<S: AsRef<str>> Sound<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Sound<&str> {
        Sound {
            fname: self.fname.as_ref(),
            volume: self.volume,
            repeat: self.repeat,
            class: self.class.as_ref().map(AsRef::as_ref),
            url: self.url.as_ref().map(AsRef::as_ref),
            priority: self.priority,
        }
    }
}

impl_partial_eq!(Sound);

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Sound<Cow<'a, str>> {
    type Error = crate::Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let fname = scanner.next_or("fname")?.expect_some("fname")?;
        let volume = scanner.next_or("v")?.expect_number()?.unwrap_or(100);
        let repeat = scanner.next_or("l")?.expect_number()?.unwrap_or_default();
        let priority = scanner.next_or("p")?.expect_number()?.unwrap_or(50);
        let class = scanner.next_or("t")?;
        let url = scanner.next_or("u")?;
        Ok(Self {
            fname,
            volume,
            repeat,
            priority,
            class,
            url,
        })
    }
}

impl FromStr for Sound {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Sound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{StringPair, format_from_pairs, parse_from_pairs};

    const AUDIO_REPETITION_PAIRS: &[StringPair<AudioRepetition>] = &[
        (AudioRepetition::Forever, "-1"),
        (AudioRepetition::Count(NonZero::new(10).unwrap()), "10"),
    ];

    #[test]
    fn fmt_audio_repetition() {
        let (actual, expected) = format_from_pairs(AUDIO_REPETITION_PAIRS);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_audio_repetition() {
        let (actual, expected) = parse_from_pairs(AUDIO_REPETITION_PAIRS);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_audio_repetition_invalid() {
        assert_eq!(
            "0".parse::<AudioRepetition>(),
            Err("0".parse::<NonZero<u32>>().unwrap_err())
        );
    }
}
