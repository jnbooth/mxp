use std::borrow::Cow;
use std::{slice, str, vec};

use super::validation::is_valid;
use crate::CaseFoldMap;
use crate::arguments::ArgumentScanner;
use crate::entity::{DecodedEntity, Entity};
use crate::{Error, ErrorKind};

/// Trait for decoding entities to values.
pub trait Decoder {
    /// Retrieves the definition for an entity by name.
    fn get_entity(&self, name: &str) -> Option<&str>;

    /// Decodes an entity by either parsing a numeric entity (e.g. `"&#32"`) or calling
    /// [`get_entity`](Self::get_entity).
    fn decode_entity(&self, name: &str) -> crate::Result<DecodedEntity<'_>> {
        let (start, radix) = match name.as_bytes() {
            [b'#', b'x', ..] => (2, 16),
            [b'#', ..] => (1, 10),
            _ => {
                return match self.get_entity(name) {
                    Some(entity) => Ok(entity.into()),
                    None if is_valid(name) => Err(Error::new(name, ErrorKind::UnknownEntity)),
                    None => Err(Error::new(name, ErrorKind::InvalidEntityName)),
                };
            }
        };
        let Ok(code) = u32::from_str_radix(&name[start..], radix) else {
            return Err(Error::new(name, ErrorKind::InvalidEntityNumber));
        };
        match char::from_u32(code) {
            Some('\0'..='\x08' | '\x0a'..='\x1f' | '\x7f'..='\u{9f}') | None => {
                // ignored per MXP standard
                Ok(DecodedEntity::Custom(""))
            }
            Some(c) => Ok(c.into()),
        }
    }

    /// Decodes an entire string, replacing all entities inside it with their values resolved by
    /// [`decode_entity`](Self::decode_entity). If the string does not contain any entities, it is
    /// returned unchanged as a borrowed string slice. Otherwise, an owned string containing the
    /// replacements is returned.
    fn decode_string<'a>(&self, mut s: &'a str) -> crate::Result<Cow<'a, str>> {
        let mut res = String::new();
        while let Some((before, rest)) = s.split_once('&') {
            if !before.is_empty() {
                res.push_str(before);
            }
            let Some((entity, after)) = rest.split_once(';') else {
                return Err(Error::new(
                    format!("&{rest}"),
                    ErrorKind::NoClosingSemicolon,
                ));
            };
            self.decode_entity(entity)?.push_to(&mut res);
            s = after;
        }
        if res.is_empty() {
            return Ok(Cow::Borrowed(s));
        }
        if !s.is_empty() {
            res.push_str(s);
        }
        Ok(Cow::Owned(res))
    }
}

impl<D: Decoder> Decoder for &D {
    fn get_entity(&self, name: &str) -> Option<&str> {
        D::get_entity(self, name)
    }

    fn decode_entity(&self, entity: &str) -> crate::Result<DecodedEntity<'_>> {
        D::decode_entity(self, entity)
    }
}

/// Fallback `Decoder` that only looks up global entities (with [`Entity::global`]).
impl Decoder for () {
    fn get_entity(&self, name: &str) -> Option<&str> {
        Entity::global(name)
    }
}

#[derive(Clone)]
pub(crate) struct Scan<'a, D: Decoder, S: AsRef<str> = Cow<'a, str>> {
    decoder: D,
    positional: slice::Iter<'a, S>,
    named: &'a CaseFoldMap<'a, S>,
}

impl<'a, D: Decoder, S: AsRef<str>> Scan<'a, D, S> {
    pub fn new(decoder: D, positional: &'a [S], named: &'a CaseFoldMap<'a, S>) -> Self {
        Self {
            decoder,
            positional: positional.iter(),
            named,
        }
    }
}

impl<'a, D: Decoder, S: AsRef<str>> ArgumentScanner for Scan<'a, D, S> {
    type Output = Cow<'a, str>;
    type RawOutput = &'a S;

    fn decode(&self, output: Self::RawOutput) -> crate::Result<Self::Output> {
        self.decoder.decode_string(output.as_ref())
    }

    fn get_named(&mut self, name: &str) -> Option<Self::RawOutput> {
        self.named.get(name)
    }

    fn get_next(&mut self) -> Option<Self::RawOutput> {
        self.positional.next()
    }
}

#[derive(Clone)]
pub(crate) struct OwnedScan<'a, D: Decoder> {
    decoder: D,
    positional: vec::IntoIter<&'a str>,
    named: CaseFoldMap<'a, &'a str>,
}

impl<'a, D: Decoder> OwnedScan<'a, D> {
    pub fn new(decoder: D, positional: Vec<&'a str>, named: CaseFoldMap<'a, &'a str>) -> Self {
        Self {
            decoder,
            positional: positional.into_iter(),
            named,
        }
    }
}

impl<'a, D: Decoder> ArgumentScanner for OwnedScan<'a, D> {
    type Output = Cow<'a, str>;
    type RawOutput = &'a str;

    fn decode(&self, output: Self::RawOutput) -> crate::Result<Self::Output> {
        self.decoder.decode_string(output.as_ref())
    }

    fn get_named(&mut self, name: &str) -> Option<Self::RawOutput> {
        self.named.remove(name)
    }

    fn get_next(&mut self) -> Option<Self::RawOutput> {
        self.positional.next()
    }
}
