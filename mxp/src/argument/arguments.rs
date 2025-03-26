use casefold::ascii::CaseFoldMap;

use super::keyword_filter::KeywordFilter;
use super::scan::{Decoder, Scan};
use crate::parser::{validate, Error, ErrorKind, Words};

/// Parsed arguments of an MXP command.
///
/// Arguments may be positional or named. For example, in the MXP command
/// `<SOUND "ouch.wav" 50 T="combat" 2 P=80>`, `"ouch.wav"`, `50`, and `2` are positional arguments,
/// and `T="combat"` and `P=80` are named arguments.
///
/// See [MXP specification: Attributes](https://www.zuggsoft.com/zmud/mxp.htm#ATTLIST).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Arguments<S: AsRef<str>> {
    positional: Vec<S>,
    named: CaseFoldMap<String, S>,
}

impl<S: AsRef<str>> Arguments<S> {
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
    pub(crate) fn find_from_attributes<'a, F: KeywordFilter, S2: AsRef<str>>(
        &'a self,
        entity: &str,
        attributes: &'a Arguments<S2>,
    ) -> Option<&'a str> {
        if let Some(named) = attributes.named.get(entity) {
            return Some(match self.named.get(entity) {
                Some(entity) => entity.as_ref(),
                None => named.as_ref(),
            });
        }
        let position = F::iter(&attributes.positional)
            .position(|attr| attr.as_ref().eq_ignore_ascii_case(entity))?;
        match F::iter(&self.positional).nth(position) {
            Some(attr) => Some(attr.as_ref()),
            None => Some(""),
        }
    }

    pub(crate) fn scan<D: Decoder>(&self, decoder: D) -> Scan<D, S> {
        Scan::new(decoder, &self.positional, &self.named)
    }

    pub(crate) fn append<'a>(&mut self, mut iter: Words<'a>) -> crate::Result<()>
    where
        S: From<&'a str>,
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
                self.named.insert(name.to_lowercase(), val.into());
            } else {
                self.positional.push(name.into());
            }
        }
        Ok(())
    }
}

impl<'a, S> TryFrom<Words<'a>> for Arguments<S>
where
    S: AsRef<str> + From<&'a str>,
{
    type Error = crate::Error;

    fn try_from(value: Words<'a>) -> crate::Result<Self> {
        let mut this = Self::new();
        this.append(value)?;
        Ok(this)
    }
}

#[cfg(test)]
mod tests {
    use casefold::ascii::CaseFold;

    use super::*;

    #[test]
    fn arguments() {
        let words = Words::new(r#"EL RName '<FONT COLOR=Red><B>' FLAG="RoomName""#);
        let args: Arguments<String> = words.try_into().unwrap();
        let expected = Arguments {
            positional: ["EL", "RName", "<FONT COLOR=Red><B>"]
                .iter()
                .map(ToString::to_string)
                .collect(),
            named: [("flag", "RoomName")]
                .iter()
                .map(|&(k, v)| (CaseFold::new(k.to_owned()), v.to_owned()))
                .collect(),
        };
        assert_eq!(args, expected);
    }
}
