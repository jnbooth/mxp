use std::borrow::Cow;
use std::fmt;
use std::num::NonZero;
use std::str::FromStr;

use crate::argument::{Decoder, ExpectArg, Scan};
use crate::parser::{Error, UnrecognizedVariant};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AudioRepetition {
    Forever,
    Count(NonZero<u32>),
}

impl Default for AudioRepetition {
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

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum AudioContinuation {
    Restart,
    #[default]
    Continue,
}

impl fmt::Display for AudioContinuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Restart => (0).fmt(f),
            Self::Continue => (1).fmt(f),
        }
    }
}

impl fmt::Debug for AudioContinuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
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
    fn parse<D, SD: AsRef<str>>(scanner: &mut Scan<'a, D, SD>) -> crate::Result<Self>
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Sound<S = String> {
    pub fname: S,
    pub volume: u8,
    pub repeats: AudioRepetition,
    pub class: Option<S>,
    pub url: Option<S>,
    pub priority: u8,
}

impl<S: AsRef<str>> Sound<S> {
    pub fn is_off(&self) -> bool {
        self.fname.as_ref().eq_ignore_ascii_case("off")
    }
}

impl Sound<&str> {
    pub fn into_owned(self) -> Sound {
        Sound {
            fname: self.fname.to_owned(),
            volume: self.volume,
            repeats: self.repeats,
            class: self.class.map(ToOwned::to_owned),
            url: self.url.map(ToOwned::to_owned),
            priority: self.priority,
        }
    }
}

impl Sound<Cow<'_, str>> {
    pub fn into_owned(self) -> Sound {
        Sound {
            fname: self.fname.into_owned(),
            volume: self.volume,
            repeats: self.repeats,
            class: self.class.map(Cow::into_owned),
            url: self.url.map(Cow::into_owned),
            priority: self.priority,
        }
    }
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for Sound<Cow<'a, str>>
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Music<S = String> {
    pub fname: S,
    pub volume: u8,
    pub repeats: AudioRepetition,
    pub class: Option<S>,
    pub url: Option<S>,
    pub continuation: AudioContinuation,
}

impl<S: AsRef<str>> Music<S> {
    pub fn is_off(&self) -> bool {
        self.fname.as_ref().eq_ignore_ascii_case("off")
    }
}

impl Music<&str> {
    pub fn into_owned(self) -> Music {
        Music {
            fname: self.fname.to_owned(),
            volume: self.volume,
            repeats: self.repeats,
            class: self.class.map(ToOwned::to_owned),
            url: self.url.map(ToOwned::to_owned),
            continuation: self.continuation,
        }
    }
}

impl Music<Cow<'_, str>> {
    pub fn into_owned(self) -> Music {
        Music {
            fname: self.fname.into_owned(),
            volume: self.volume,
            repeats: self.repeats,
            class: self.class.map(Cow::into_owned),
            url: self.url.map(Cow::into_owned),
            continuation: self.continuation,
        }
    }
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for Music<Cow<'a, str>>
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let args = SoundOrMusic::parse(&mut scanner)?;
        Ok(Self {
            fname: args.fname,
            volume: args.volume,
            repeats: args.repeats,
            class: args.class,
            url: args.url,
            continuation: scanner
                .next_or("C")?
                .and_then(|continuation| continuation.as_ref().parse().ok())
                .unwrap_or_default(),
        })
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
