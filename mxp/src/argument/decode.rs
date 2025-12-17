use std::borrow::Cow;
use std::str;

use super::arguments::Arguments;
use super::keyword_filter::KeywordFilter;
use super::scan::Decoder;
use crate::element::Element;
use crate::entity::EntityMap;
use crate::parser::{Error, ErrorKind};

fn decode_amps<'a, F>(mut s: &str, mut f: F) -> crate::Result<Cow<'_, str>>
where
    F: FnMut(&str) -> crate::Result<Option<&'a str>>,
{
    let mut res = String::new();
    while let Some(start) = s.find('&') {
        if start > 0 {
            res.push_str(&s[..start]);
        }
        s = &s[start..];
        let end = s
            .find(';')
            .ok_or_else(|| Error::new(s, ErrorKind::NoClosingSemicolon))?;
        res.push_str(f(&s[1..end])?.unwrap_or(&s[..=end]));
        s = &s[end + 1..];
    }
    if res.is_empty() {
        return Ok(Cow::Borrowed(s));
    }
    if !s.is_empty() {
        res.push_str(s);
    }
    Ok(Cow::Owned(res))
}

impl Decoder for EntityMap {
    fn decode<'a, F: KeywordFilter>(&self, s: &'a str) -> crate::Result<Cow<'a, str>> {
        decode_amps(s, |entity| self.decode_entity(entity))
    }
}

/// A [`Decoder`] that uses entity definitions provided by an [`Element`].
///
/// This `struct` is created by [`State::decode_element`](crate::State::decode_element).
/// See its documentation for more.
#[derive(Debug, PartialEq, Eq)]
pub struct ElementDecoder<'a, S: AsRef<str>> {
    pub(crate) element: &'a Element,
    pub(crate) entities: &'a EntityMap,
    pub(crate) args: &'a Arguments<S>,
}

impl<S: AsRef<str>> Copy for ElementDecoder<'_, S> {}

impl<S: AsRef<str>> Clone for ElementDecoder<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: AsRef<str>> Decoder for ElementDecoder<'_, S> {
    fn decode<'a, F: KeywordFilter>(&self, s: &'a str) -> crate::Result<Cow<'a, str>> {
        decode_amps(s, |entity| {
            if entity == "text" {
                return Ok(None);
            }
            match self
                .args
                .find_from_attributes::<F, _>(entity, &self.element.attributes)
            {
                Some(attr) => Ok(Some(attr)),
                None => self.entities.decode_entity(entity),
            }
        })
    }
}
