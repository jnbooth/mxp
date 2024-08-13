use std::slice;

use super::element_map::{ElementComponent, ElementMap};
use super::entity_map::{ElementDecoder, EntityMap};
use crate::argument::scan::{Decoder, Scan};
use crate::argument::{Arguments, Keyword};
use crate::entity::{Action, Element, ElementItem};
use crate::parser::{Error as MxpError, ParseError, Words};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    elements: ElementMap,
    entities: EntityMap,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.entities.clear();
    }

    pub fn get_component(&self, name: &str) -> Result<ElementComponent, ParseError> {
        self.elements.get_component(name)
    }

    pub fn get_entity(&self, name: &str) -> Result<Option<&str>, ParseError> {
        self.entities.get(name)
    }

    pub fn decode_args<'a>(&self, args: &'a mut Arguments) -> Scan<'a, &EntityMap> {
        args.scan(&self.entities)
    }

    pub fn decode_element<'a>(
        &'a self,
        element: &'a Element,
        args: &'a Arguments,
    ) -> DecodeElement<'a, ElementDecoder<'a>> {
        let decoder = self.entities.element_decoder(element, args);
        DecodeElement {
            decoder,
            items: element.items.iter(),
        }
    }

    pub fn define(&mut self, tag: &str) -> Result<(), ParseError> {
        let mut words = Words::new(tag);

        let definition = words.validate_next_or(MxpError::InvalidDefinition)?;
        let name = words.validate_next_or(MxpError::InvalidElementName)?;
        match_ci! {definition,
            "element" | "el" => self.define_element(name, words),
            "entity" | "en" => self.define_entity(name, words),
            "attlist" | "att" => self.define_attributes(name, words),
            _ => Err(ParseError::new(definition, MxpError::InvalidDefinition))
        }
    }

    fn define_element(&mut self, name: &str, words: Words) -> Result<(), ParseError> {
        let args = Arguments::parse(words)?;
        if args.has_keyword(Keyword::Delete) {
            self.elements.remove(&name);
            return Ok(());
        }
        let el = Element::parse(name.to_owned(), args.scan(&self.entities))?;
        self.elements.insert(name.to_owned(), el);
        Ok(())
    }

    fn define_entity(&mut self, key: &str, mut words: Words) -> Result<(), ParseError> {
        if EntityMap::global(key).is_some() {
            return Err(ParseError::new(key, MxpError::CannotRedefineEntity));
        }
        match words.next() {
            Some(body) // once told me
                if !words.any(|word| {
                    word.eq_ignore_ascii_case("delete") || word.eq_ignore_ascii_case("remove")
                }) =>
            {
                let value = self.entities.decode(body)?.into_owned();
                self.entities.insert(key.to_owned(), value);
            }
            _ => {
                self.entities.remove(key);
            }
        };
        Ok(())
    }

    fn define_attributes(&mut self, key: &str, words: Words) -> Result<(), ParseError> {
        self.elements
            .get_mut(key)
            .ok_or_else(|| ParseError::new(key, MxpError::UnknownElementInAttlist))?
            .attributes
            .append(words)
    }
}

pub struct DecodeElement<'a, D> {
    decoder: D,
    items: slice::Iter<'a, ElementItem>,
}

impl<'a, D: Decoder + Copy> Iterator for DecodeElement<'a, D> {
    type Item = Result<Action<D::Output<'a>>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let item: &'a ElementItem = self.items.next()?;
        let scanner = item.arguments.scan(self.decoder);
        Some(Action::new(item.atom.action, scanner))
    }
}
