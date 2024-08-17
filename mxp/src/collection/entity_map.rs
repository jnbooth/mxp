use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

use super::decode::{decode_amps, CHARS};
use super::variable_map::VariableMap;
use crate::argument::scan::Decoder;
use crate::argument::{Arguments, KeywordFilter};
use crate::entity::Element;

use crate::parser::{Error, ErrorKind};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityMap {
    inner: VariableMap,
    has_globals: bool,
}

impl Deref for EntityMap {
    type Target = VariableMap;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for EntityMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl EntityMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.has_globals = false;
    }

    pub fn decode_entity(&self, key: &str) -> crate::Result<Option<&str>> {
        if !key.starts_with('#') {
            return Ok(self.inner.get(key).map(|entity| entity.value.as_ref()));
        }
        let id = match key.strip_prefix('x') {
            Some(hex) => u8::from_str_radix(hex, 16),
            None => key.parse::<u8>(),
        }
        .map_err(|_| Error::new(key, ErrorKind::InvalidEntityNumber))?;
        let id = id as usize;
        match CHARS.get(id..=id) {
            None | Some("\x00") => Err(Error::new(key, ErrorKind::DisallowedEntityNumber)),
            some => Ok(some),
        }
    }

    pub fn element_decoder<'a>(
        &'a self,
        element: &'a Element,
        args: &'a Arguments,
    ) -> ElementDecoder {
        ElementDecoder {
            element,
            entities: self,
            args,
        }
    }
}

impl Decoder for EntityMap {
    type Output<'a> = Cow<'a, str>;

    fn decode<'a, F: KeywordFilter>(&self, s: &'a str) -> crate::Result<Self::Output<'a>> {
        decode_amps(s, |entity| self.decode_entity(entity))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ElementDecoder<'a> {
    element: &'a Element,
    entities: &'a EntityMap,
    args: &'a Arguments,
}

impl<'d> Decoder for ElementDecoder<'d> {
    type Output<'a> = Cow<'a, str>;

    fn decode<'a, F: KeywordFilter>(&self, s: &'a str) -> crate::Result<Self::Output<'a>> {
        decode_amps(s, |entity| {
            if entity == "text" {
                return Ok(None);
            }
            match self
                .element
                .attributes
                .find_attribute::<F>(entity, self.args)
            {
                Some(attr) => Ok(Some(attr)),
                None => self.entities.decode_entity(entity),
            }
        })
    }
}
