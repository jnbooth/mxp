use std::borrow::Cow;
use std::collections::hash_map;
use std::fmt;
use std::str::FromStr;

use uncased::Uncased;

use super::iter::{Named, Positional};
use crate::CaseFoldMap;
use crate::parse::{ArgumentParser, OwnedScan, Scan, validate};
use crate::{Error, ErrorKind};

/// Parsed arguments of an MXP command.
///
/// Arguments may be positional or named. For example, in the MXP command
/// `<SOUND "ouch.wav" 50 T="combat" 2 P=80>`, `"ouch.wav"`, `50`, and `2` are positional arguments,
/// and `T="combat"` and `P=80` are named arguments.
///
/// Note: Although `Arguments` has a public API, it is unlikely that a client will need to do
/// anything with `Arguments` beyond parsing them from text and passing them to mxp functions.
///
/// See [MXP specification: Attributes](https://www.zuggsoft.com/zmud/mxp.htm#ATTLIST).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arguments<'a, S = &'a str> {
    positional: Vec<S>,
    named: CaseFoldMap<'a, S>,
}

impl<S> Default for Arguments<'_, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S> Arguments<'a, S> {
    /// Constructs a new, empty `Arguments<S>`.
    pub fn new() -> Self {
        Self {
            positional: Vec::new(),
            named: CaseFoldMap::new(),
        }
    }

    /// Returns `true` if there are no arguments.
    pub fn is_empty(&self) -> bool {
        self.positional.is_empty() && self.named.is_empty()
    }

    /// Removes all arguments.
    pub fn clear(&mut self) {
        self.named.clear();
        self.positional.clear();
    }

    /// Finds the value of a positional argument at a specified index.
    pub fn at(&self, i: usize) -> Option<&S> {
        self.positional.get(i)
    }

    /// Finds the value of a named argument associated with the specified key.
    pub fn get(&self, name: &str) -> Option<&S> {
        self.named.get(name)
    }

    /// Iterator that visits all named arguments in arbitrary order. The iterator element type is
    /// `(&'a str, &'a S)`.
    pub fn named(&self) -> Named<'_, S> {
        Named {
            inner: self.named.iter(),
        }
    }

    /// Iterator that visits all positional arguments in sequence. The iterator element type is
    /// `&'a S`.
    pub fn positional(&self) -> Positional<'_, S> {
        Positional {
            inner: self.positional.iter(),
        }
    }

    /// Adds a new positional argument. Mainly useful for server-side implementations.
    ///
    /// Note: this function does not escape special characters. If the value is unescaped, you
    /// should use [`html_escape::encode_double_quoted_attribute`] on it before inerting
    /// it.
    pub fn push(&mut self, arg: S) {
        self.positional.push(arg);
    }
    /// Adds a new named argument argument, returning the value that was previously associated with
    /// that name if one existed. Mainly useful for server-side implementations.
    ///
    /// Note: this function does not escape special characters. If the value is unescaped, you
    /// should use [`html_escape::encode_double_quoted_attribute`] on it before inerting
    /// it.
    pub fn insert<K>(&mut self, name: K, arg: S) -> Option<S>
    where
        K: Into<Cow<'a, str>>,
    {
        self.named.insert(name.into(), arg)
    }

    pub(crate) fn keys(&self) -> hash_map::Keys<'_, Uncased<'a>, S> {
        self.named.keys()
    }
}

impl<S: AsRef<str>> Arguments<'_, S> {
    pub(crate) fn scan(&self) -> Scan<'_, S> {
        Scan::new(&self.positional, &self.named)
    }
}

impl<S> Extend<S> for Arguments<'_, S> {
    /// Adds new positional arguments or keywords. Mainly useful for server-side implementations.
    ///
    /// Note: this function does not escape special characters. If the value is unescaped, you
    /// should use [`html_escape::encode_double_quoted_attribute`] on it before inerting
    /// it.
    fn extend<I: IntoIterator<Item = S>>(&mut self, iter: I) {
        self.positional.extend(iter);
    }
}

impl<'a, K, S> Extend<(K, S)> for Arguments<'a, S>
where
    K: Into<Cow<'a, str>>,
{
    /// Adds new named arguments. Mainly useful for server-side implementations.
    ///
    /// Note: this function does not escape special characters. If the value is unescaped, you
    /// should use [`html_escape::encode_double_quoted_attribute`] on it before inerting
    /// it.
    fn extend<T: IntoIterator<Item = (K, S)>>(&mut self, iter: T) {
        self.named.extend(iter);
    }
}

impl<'a> Arguments<'a> {
    /// Parses arguments from a string slice without cloning the data.
    pub fn parse(source: &'a str) -> crate::Result<Self> {
        ArgumentParser::new(source).try_into()
    }

    pub(crate) fn into_scan(self) -> OwnedScan<'a> {
        OwnedScan::new(self.positional, self.named)
    }
}

impl<'a, 'b, S> TryFrom<ArgumentParser<'b>> for Arguments<'a, S>
where
    S: From<&'b str> + Into<Cow<'a, str>>,
{
    type Error = Error;

    fn try_from(args: ArgumentParser<'b>) -> crate::Result<Self> {
        let generous_size_guess = args.size_hint().1.unwrap();
        let mut positional = Vec::with_capacity(generous_size_guess);
        let mut named = CaseFoldMap::with_capacity(generous_size_guess);
        for entry in args {
            let (name, value) = entry?;
            if let Some(value) = value {
                validate(name, ErrorKind::InvalidArgumentName)?;
                named.insert(S::from(name).into(), S::from(value));
            } else {
                positional.push(S::from(name));
            }
        }
        Ok(Self { positional, named })
    }
}

impl<S: AsRef<str>> fmt::Display for Arguments<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::display::{DelimAfterFirst, MaybeQuote};

        let delim = DelimAfterFirst::new(" ");
        for positional in &self.positional {
            write!(f, "{delim}{}", MaybeQuote(positional.as_ref()))?;
        }
        for (k, v) in &self.named {
            write!(f, "{delim}{k}={}", MaybeQuote(v.as_ref()))?;
        }

        Ok(())
    }
}

impl FromStr for Arguments<'static, String> {
    type Err = Error;

    fn from_str(s: &str) -> crate::Result<Self> {
        ArgumentParser::new(s).try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments() {
        let args: Arguments =
            ArgumentParser::new("  EL      RName  '<FONT COLOR=Red><B>' FLAG=\"RoomName\"  ")
                .try_into()
                .unwrap();
        let expected = Arguments {
            positional: vec!["EL", "RName", "<FONT COLOR=Red><B>"],
            named: [("flag", "RoomName")].iter().copied().collect(),
        };
        assert_eq!(args, expected);
    }
}
