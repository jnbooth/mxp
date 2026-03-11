use std::borrow::Cow;
use std::fmt;
use std::num::NonZero;
use std::str::FromStr;

use crate::parse::{Decoder, Error, ExpectArg as _, Scan, StringVariant, UnrecognizedVariant};

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
    type Err = <NonZero<u32> as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-1" {
            return Ok(Self::Forever);
        }
        s.parse().map(Self::Count)
    }
}

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

impl fmt::Display for AudioContinuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self as u8).fmt(f)
    }
}

impl fmt::Debug for AudioContinuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl StringVariant for AudioContinuation {
    type Variant = &'static str;
    const VARIANTS: &[&'static str] = &["0", "1"];
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct SoundOrMusic<S> {
    fname: S,
    volume: u8,
    repeats: AudioRepetition,
    class: Option<S>,
    url: Option<S>,
}

impl<'a> SoundOrMusic<Cow<'a, str>> {
    fn parse<D>(scanner: &mut Scan<'a, D>) -> crate::Result<Self>
    where
        D: Decoder,
    {
        Ok(Self {
            fname: scanner.next()?.expect_some("fname")?,
            volume: scanner.next_or("V")?.expect_number()?.unwrap_or(100),
            repeats: scanner.next_or("L")?.expect_number()?.unwrap_or_default(),
            class: scanner.next_or("C")?,
            url: scanner.next_or("U")?,
        })
    }
}

#[allow(clippy::ref_option)]
fn form_uri<'a, S: AsRef<str>>(fname: &'a S, url: &Option<S>) -> Cow<'a, str> {
    let fname = fname.as_ref();
    let Some(url) = url else {
        return Cow::Borrowed(fname);
    };
    let url = url.as_ref();
    let infix = if url.ends_with('/') { "" } else { "/" };
    let uri = format!("{url}{infix}{fname}");
    Cow::Owned(uri)
}

/// Sound triggers are WAV format files intended for sound effects.
///
/// See [MXP specification: `<SOUND>`](https://www.zuggsoft.com/zmud/mxp.htm#MSP%20Compatibility)
/// and the [MSP (Mud Sound Protocol) specification](https://www.zuggsoft.com/zmud/msp.htm).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Sound<S = String> {
    /// File name. May contain wildcards. If no extension is specified, ".wav" should be assumed.
    pub fname: S,
    /// Volume between 0 and 100.
    pub volume: u8,
    /// Repeat behavior.
    pub repeats: AudioRepetition,
    /// Type of sound, e.g. combat, zone, death, clan. Case-insensitive. This parameter was
    /// intended to provide a way to group sounds into subfolders within the main sound directory.
    pub class: Option<S>,
    /// Specifies the URL of the sound file. This allows downloading files from the MUD server.
    /// Client should always look in local directories first, and only download the file if it's
    /// not available locally.
    pub url: Option<S>,
    /// This parameter applies when some sound is playing and another request arrives. Then, if new
    /// request has higher (but NOT equal) priority than the one that's currently being played, old
    /// sound must be stopped and the new sound starts playing instead. In the case of a tie, the
    /// sound that is already playing wins.
    pub priority: u8,
}

impl<S: AsRef<str>> Sound<S> {
    /// Returns `true` if this command is a `<SOUND OFF>` command, causing sounds to stop rather
    /// than triggering a sound.
    pub fn is_off(&self) -> bool {
        self.fname.as_ref().eq_ignore_ascii_case("off")
    }

    /// Combines `self.url` and `self.fname` into a single URI.
    pub fn uri(&self) -> Cow<'_, str> {
        form_uri(&self.fname, &self.url)
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
            repeats: self.repeats,
            class: self.class.map(&mut f),
            url: self.url.map(f),
            priority: self.priority,
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
            repeats: self.repeats,
            class: self.class.as_ref().map(AsRef::as_ref),
            url: self.url.as_ref().map(AsRef::as_ref),
            priority: self.priority,
        }
    }
}

impl_partial_eq!(Sound);

impl<'a, D> TryFrom<Scan<'a, D>> for Sound<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        let args = SoundOrMusic::parse(&mut scanner)?;
        Ok(Self {
            fname: args.fname,
            volume: args.volume,
            repeats: args.repeats,
            class: args.class,
            url: args.url,
            priority: scanner.next_or("P")?.expect_number()?.unwrap_or(50),
        })
    }
}

impl FromStr for Sound {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Sound)
    }
}

/// Music triggers are typically background MID (MIDI) files. Only one music trigger can be active
/// at once.
///
/// See [MXP specification: `<MUSIC>`](https://www.zuggsoft.com/zmud/mxp.htm#MSP%20Compatibility)
/// and the [MSP (Mud Sound Protocol) specification](https://www.zuggsoft.com/zmud/msp.htm).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Music<S = String> {
    /// File name. May contain wildcards. If no extension is specified, ".midi" should be assumed.
    pub fname: S,
    /// Volume between 0 and 100.
    pub volume: u8,
    /// Repeat behavior.
    pub repeats: AudioRepetition,
    /// Type of music, e.g. combat, zone, death, clan. Case-insensitive. This parameter was
    /// intended to provide a way to group music into subfolders within the main music directory.
    pub class: Option<S>,
    /// Specifies the URL of the music file. This allows downloading files from the MUD server.
    /// Client should always look in local directories first, and only download the file if it's
    /// not available locally.
    pub url: Option<S>,
    /// File behavior if the server requests it should play again while it is already playing.
    pub continuation: AudioContinuation,
}

impl<S: AsRef<str>> Music<S> {
    /// Returns `true` if this command is a `<MUSIC OFF>` command, causing sounds to stop rather
    /// than triggering a sound.
    pub fn is_off(&self) -> bool {
        self.fname.as_ref().eq_ignore_ascii_case("off")
    }

    /// Combines `self.url` and `self.fname` into a single URI.
    pub fn uri(&self) -> Cow<'_, str> {
        form_uri(&self.fname, &self.url)
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
            repeats: self.repeats,
            class: self.class.map(&mut f),
            url: self.url.map(f),
            continuation: self.continuation,
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
            repeats: self.repeats,
            class: self.class.as_ref().map(AsRef::as_ref),
            url: self.url.as_ref().map(AsRef::as_ref),
            continuation: self.continuation,
        }
    }
}

impl_partial_eq!(Music);

impl<'a, D> TryFrom<Scan<'a, D>> for Music<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        let args = SoundOrMusic::parse(&mut scanner)?;
        Ok(Self {
            fname: args.fname,
            volume: args.volume,
            repeats: args.repeats,
            class: args.class,
            url: args.url,
            continuation: scanner
                .next_or("C")?
                .and_then(|continuation| continuation.parse().ok())
                .unwrap_or_default(),
        })
    }
}

impl FromStr for Music {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Music)
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
