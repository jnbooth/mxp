use std::borrow::Cow;
use std::collections::hash_map;
use std::fmt;
use std::str::FromStr;

use uncased::Uncased;

use super::iter::{Named, Positional};
use crate::CaseFoldMap;
use crate::keyword::KeywordFilter;
use crate::parse::{Decoder, OwnedScan, Scan, Words, validate};
use crate::{Error, ErrorKind};

/// Parsed arguments of an MXP command.
///
/// Arguments may be positional or named. For example, in the MXP command
/// `<SOUND "ouch.wav" 50 T="combat" 2 P=80>`, `"ouch.wav"`, `50`, and `2` are positional arguments,
/// and `T="combat"` and `P=80` are named arguments.
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

    /// Returns `true` if there are no parsed arguments.
    pub fn is_empty(&self) -> bool {
        self.positional.is_empty() && self.named.is_empty()
    }

    /// Finds the positional value at a specified index.
    pub fn at(&self, i: usize) -> Option<&S> {
        self.positional.get(i)
    }

    /// Finds the value associated with a named key.
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

impl<'a, S: AsRef<str>> Arguments<'a, S> {
    /// Finds the value of an entity, using an element's attribute list to identify arguments
    /// and provide default values.
    pub(crate) fn find_from_attributes<K: KeywordFilter>(
        &'a self,
        entity: &str,
        attributes: &'a Arguments<'static, String>,
    ) -> Option<&'a str> {
        if let Some(named) = attributes.named.get(entity) {
            return match self.named.get(entity) {
                Some(entity) => Some(entity.as_ref()),
                None => Some(named),
            };
        }
        let position =
            K::iter(&attributes.positional).position(|attr| attr.eq_ignore_ascii_case(entity))?;
        match K::iter(&self.positional).nth(position) {
            Some(attr) => Some(attr.as_ref()),
            None => Some(""),
        }
    }

    pub(crate) fn scan<D: Decoder>(&self, decoder: D) -> Scan<'_, D, S> {
        Scan::new(decoder, &self.positional, &self.named)
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.named.shrink_to_fit();
        self.positional.shrink_to_fit();
    }

    fn append_inner<'b, T>(&mut self, mut iter: Words<'b>) -> crate::Result<()>
    where
        T: From<&'b str> + Into<S> + Into<Uncased<'a>>,
    {
        let generous_size_guess = iter.size_hint().1.unwrap();
        self.named.reserve(generous_size_guess);
        self.positional.reserve(generous_size_guess);
        while let Some(name) = iter.next() {
            if name == "/" {
                if iter.next().is_none() {
                    return Ok(());
                }
                return Err(Error::new(name, ErrorKind::InvalidArgumentName));
            }
            if iter.as_str().starts_with('=') {
                validate(name, ErrorKind::InvalidArgumentName)?;
                iter.next();
                let val = iter
                    .next()
                    .ok_or_else(|| Error::new(format!("{name}="), ErrorKind::EmptyArgument))?;
                self.named.insert(T::from(name), T::from(val).into());
            } else {
                self.positional.push(T::from(name).into());
            }
        }
        Ok(())
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
        Words::new(source).try_into()
    }

    pub(crate) fn append(&mut self, iter: Words<'a>) -> crate::Result<()> {
        self.append_inner::<&str>(iter)
    }

    pub(crate) fn into_scan<D: Decoder>(self, decoder: D) -> OwnedScan<'a, D> {
        OwnedScan::new(decoder, self.positional, self.named)
    }
}

impl<'a> Arguments<'a, Cow<'a, str>> {
    pub(crate) fn append(&mut self, iter: Words<'a>) -> crate::Result<()> {
        self.append_inner::<&str>(iter)
    }
}

impl Arguments<'static, String> {
    pub(crate) fn append(&mut self, iter: Words<'_>) -> crate::Result<()> {
        self.append_inner::<String>(iter)?;
        self.shrink_to_fit();
        Ok(())
    }
}

impl<'a> TryFrom<Words<'a>> for Arguments<'a> {
    type Error = Error;

    fn try_from(value: Words<'a>) -> crate::Result<Self> {
        let mut this = Self::new();
        this.append(value)?;
        Ok(this)
    }
}

impl<'a> TryFrom<Words<'a>> for Arguments<'a, Cow<'a, str>> {
    type Error = Error;

    fn try_from(value: Words<'a>) -> crate::Result<Self> {
        let mut this = Self::new();
        this.append(value)?;
        Ok(this)
    }
}

impl TryFrom<Words<'_>> for Arguments<'static, String> {
    type Error = Error;

    fn try_from(value: Words<'_>) -> crate::Result<Self> {
        let mut this = Self::new();
        this.append(value)?;
        Ok(this)
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
        Words::new(s).try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments() {
        let words = Words::new("EL RName '<FONT COLOR=Red><B>' FLAG=\"RoomName\"");
        let args: Arguments = words.try_into().unwrap();
        let expected = Arguments {
            positional: vec!["EL", "RName", "<FONT COLOR=Red><B>"],
            named: [("flag", "RoomName")].iter().copied().collect(),
        };
        assert_eq!(args, expected);
    }
}
