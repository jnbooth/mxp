use std::borrow::Cow;

use super::keyword_filter::KeywordFilter;
use super::scan::{Decoder, Scan};
use crate::collections::CaseFoldMap;
use crate::parser::{Error, ErrorKind, Words, validate};

/// Parsed arguments of an MXP command.
///
/// Arguments may be positional or named. For example, in the MXP command
/// `<SOUND "ouch.wav" 50 T="combat" 2 P=80>`, `"ouch.wav"`, `50`, and `2` are positional arguments,
/// and `T="combat"` and `P=80` are named arguments.
///
/// See [MXP specification: Attributes](https://www.zuggsoft.com/zmud/mxp.htm#ATTLIST).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Arguments<'a> {
    positional: Vec<Cow<'a, str>>,
    named: CaseFoldMap<'a, Cow<'a, str>>,
}

impl<'a> Arguments<'a> {
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

    /// Finds the value of an entity, using an element's attribute list to identify arguments
    /// and provide default values.
    pub(crate) fn find_from_attributes<F>(
        &'a self,
        entity: &str,
        attributes: &'a Arguments<'a>,
    ) -> Option<&'a str>
    where
        F: KeywordFilter,
    {
        if let Some(named) = attributes.named.get(entity) {
            return Some(match self.named.get(entity) {
                Some(entity) => entity,
                None => named,
            });
        }
        let position =
            F::iter(&attributes.positional).position(|attr| attr.eq_ignore_ascii_case(entity))?;
        match F::iter(&self.positional).nth(position) {
            Some(attr) => Some(attr),
            None => Some(""),
        }
    }

    pub(crate) fn scan<D: Decoder>(&self, decoder: D) -> Scan<'_, D> {
        Scan::new(decoder, &self.positional, &self.named)
    }

    pub(crate) fn extend<'b, S>(&mut self, mut iter: Words<'b>) -> crate::Result<()>
    where
        S: From<&'b str> + Into<Cow<'a, str>>,
    {
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
                    .ok_or_else(|| Error::new(name, ErrorKind::NoArgument))?;
                self.named.insert(S::from(name).into(), S::from(val).into());
            } else {
                self.positional.push(S::from(name).into());
            }
        }
        Ok(())
    }
}

impl<'a> TryFrom<Words<'a>> for Arguments<'a> {
    type Error = crate::Error;

    fn try_from(value: Words<'a>) -> crate::Result<Self> {
        let mut this = Self::new();
        this.extend::<&str>(value)?;
        Ok(this)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments() {
        let words = Words::new(r#"EL RName '<FONT COLOR=Red><B>' FLAG="RoomName""#);
        let args: Arguments = words.try_into().unwrap();
        let expected = Arguments {
            positional: ["EL", "RName", "<FONT COLOR=Red><B>"]
                .iter()
                .map(|&arg| arg.into())
                .collect(),
            named: [("flag", "RoomName")]
                .iter()
                .map(|&(k, v)| (k, v.into()))
                .collect(),
        };
        assert_eq!(args, expected);
    }
}
