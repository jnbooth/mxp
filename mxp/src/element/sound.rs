use crate::argument::{Decoder, ExpectArg, Scan};
use crate::parser::{Error, UnrecognizedVariant};
use std::borrow::Cow;
use std::fmt::{self, Debug, Display, Formatter};
use std::num::NonZero;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AudioRepetition {
    Forever,
    Count(NonZero<u32>),
}

impl Default for AudioRepetition {
    fn default() -> Self {
        Self::Count(NonZero::new(1).unwrap())
    }
}

impl Display for AudioRepetition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Forever => f.write_str("-1"),
            Self::Count(amount) => Display::fmt(amount, f),
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AudioContinuation {
    Restart,
    Continue,
}

impl Default for AudioContinuation {
    fn default() -> Self {
        Self::Continue
    }
}

impl Display for AudioContinuation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Restart => f.write_str("0"),
            Self::Continue => f.write_str("1"),
        }
    }
}

impl Debug for AudioContinuation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        self.fname.as_ref().eq_ignore_ascii_case("Off")
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

impl<'a> Sound<Cow<'a, str>> {
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

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Sound<Cow<'a, str>> {
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        self.fname.as_ref().eq_ignore_ascii_case("Off")
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

impl<'a> Music<Cow<'a, str>> {
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

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Music<Cow<'a, str>> {
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
    use crate::test_utils::{const_nonzero, format_from_pairs, parse_from_pairs, StringPair};

    const AUDIO_REPETITION_PAIRS: &[StringPair<AudioRepetition>] = &[
        (AudioRepetition::Forever, "-1"),
        (AudioRepetition::Count(const_nonzero!(10)), "10"),
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
