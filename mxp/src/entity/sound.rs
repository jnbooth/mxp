use crate::argument::scan::{Decoder, Scan};
use crate::parser::{Error, ErrorKind, UnrecognizedVariant};
use enumeration::Enum;
use std::borrow::Cow;
use std::fmt::{self, Debug, Display, Formatter};
use std::num::NonZeroU32;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AudioRepetition {
    Forever,
    Count(NonZeroU32),
}

impl Default for AudioRepetition {
    fn default() -> Self {
        const_non_zero!(NON_ZERO_ONE, NonZeroU32, 1);
        Self::Count(NON_ZERO_ONE)
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
    type Err = <NonZeroU32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-1" {
            return Ok(Self::Forever);
        }
        s.parse().map(Self::Count)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
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

impl<S> SoundOrMusic<S> {
    fn parse<'a, D>(scanner: &mut Scan<'a, D>) -> crate::Result<Self>
    where
        D: Decoder<Output<'a> = S>,
    {
        Ok(Self {
            fname: scanner
                .next()?
                .ok_or_else(|| Error::new("fname", ErrorKind::IncompleteArguments))?,
            volume: scanner.next_number_or("V")?.unwrap_or(100),
            repeats: scanner.next_number_or("L")?.unwrap_or_default(),
            class: scanner.next_or("C")?,
            url: scanner.next_or("U")?,
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sound<S = String> {
    fname: S,
    volume: u8,
    repeats: AudioRepetition,
    class: Option<S>,
    url: Option<S>,
    priority: u8,
}

impl<S: AsRef<str>> Sound<S> {
    pub fn is_off(&self) -> bool {
        self.fname.as_ref().eq_ignore_ascii_case("Off")
    }
}

impl Sound<&str> {
    pub fn into_owned(&self) -> Sound {
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

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for Sound<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        let args = SoundOrMusic::parse(&mut scanner)?;
        Ok(Self {
            fname: args.fname,
            volume: args.volume,
            repeats: args.repeats,
            class: args.class,
            url: args.url,
            priority: scanner.next_number_or("P")?.unwrap_or(50),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Music<S = String> {
    fname: S,
    volume: u8,
    repeats: AudioRepetition,
    class: Option<S>,
    url: Option<S>,
    continuation: AudioContinuation,
}

impl<S: AsRef<str>> Music<S> {
    pub fn is_off(&self) -> bool {
        self.fname.as_ref().eq_ignore_ascii_case("Off")
    }
}

impl Music<&str> {
    pub fn into_owned(&self) -> Music {
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

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for Music<D::Output<'a>> {
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
                .and_then(|continuation| continuation.as_ref().parse().ok())
                .unwrap_or_default(),
        })
    }
}
