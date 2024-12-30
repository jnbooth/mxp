use std::borrow::Cow;

use super::arguments::Arguments;
use super::keyword_filter::KeywordFilter;
use super::scan::Decoder;
use crate::element::Element;
use crate::entity::EntityMap;
use crate::parser::{Error, ErrorKind};
use std::str;

fn decode_amps<'a, F>(mut s: &str, mut f: F) -> crate::Result<Cow<str>>
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

#[derive(Debug, PartialEq, Eq)]
pub struct ElementDecoder<'a, S> {
    pub(crate) element: &'a Element,
    pub(crate) entities: &'a EntityMap,
    pub(crate) args: &'a Arguments<S>,
}

impl<'a, S> Copy for ElementDecoder<'a, S> {}

impl<'a, S> Clone for ElementDecoder<'a, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'d, S: AsRef<str>> Decoder for ElementDecoder<'d, S> {
    fn decode<'a, F: KeywordFilter>(&self, s: &'a str) -> crate::Result<Cow<'a, str>> {
        decode_amps(s, |entity| {
            if entity == "text" {
                return Ok(None);
            }
            match self
                .element
                .attributes
                .find_attribute::<F, S>(entity, self.args)
            {
                Some(attr) => Ok(Some(attr)),
                None => self.entities.decode_entity(entity),
            }
        })
    }
}
