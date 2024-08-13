use std::slice;

use super::element_map::{ElementComponent, ElementMap};
use super::entity_map::{ElementDecoder, EntityMap};
use super::line_tags::{LineTagUpdate, LineTags};
use crate::argument::scan::{Decoder, Scan};
use crate::argument::{Arguments, Keyword};
use crate::entity::{Action, Element, ElementItem, Mode};
use crate::parser::{Error, ErrorKind, Words};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    elements: ElementMap,
    entities: EntityMap,
    line_tags: LineTags,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.entities.clear();
    }

    pub fn get_component(&self, name: &str) -> crate::Result<ElementComponent> {
        self.elements.get_component(name)
    }

    pub fn get_entity(&self, name: &str) -> crate::Result<Option<&str>> {
        self.entities.get(name)
    }

    pub fn get_line_tag(&self, mode: Mode) -> Option<&Element> {
        self.line_tags.get(mode.0 as usize, &self.elements)
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

    pub fn define(&mut self, tag: &str) -> crate::Result<()> {
        let mut words = Words::new(tag);

        let definition = words.validate_next_or(ErrorKind::InvalidDefinition)?;
        if definition.eq_ignore_ascii_case("tag") {
            return self.define_line_tag(words);
        }
        let name = words.validate_next_or(ErrorKind::InvalidElementName)?;
        match_ci! {definition,
            "ELEMENT" | "EL" => self.define_element(name, words),
            "ENTITY" | "EN" => self.define_entity(name, words),
            "ATTLIST" | "ATT" => self.define_attributes(name, words),
            _ => Err(Error::new(definition, ErrorKind::InvalidDefinition))
        }
    }

    fn define_element(&mut self, name: &str, words: Words) -> crate::Result<()> {
        let args = Arguments::parse(words)?;
        if args.has_keyword(Keyword::Delete) {
            self.elements.remove(&name);
            return Ok(());
        }
        let el = Element::parse(name.to_owned(), args.scan(&self.entities))?;
        if let Some(tag) = el.tag {
            self.line_tags.set(tag.get() as usize, el.name.clone());
        }
        self.elements.insert(name.to_owned(), el);
        Ok(())
    }

    pub fn define_line_tag(&mut self, words: Words) -> crate::Result<()> {
        let update = LineTagUpdate::parse(words, &self.entities)?;
        self.line_tags.update(update, &mut self.elements);
        Ok(())
    }

    fn define_entity(&mut self, key: &str, mut words: Words) -> crate::Result<()> {
        if EntityMap::global(key).is_some() {
            return Err(Error::new(key, ErrorKind::CannotRedefineEntity));
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

    fn define_attributes(&mut self, key: &str, words: Words) -> crate::Result<()> {
        self.elements
            .get_mut(key)
            .ok_or_else(|| Error::new(key, ErrorKind::UnknownElementInAttlist))?
            .attributes
            .append(words)
    }
}

pub struct DecodeElement<'a, D> {
    decoder: D,
    items: slice::Iter<'a, ElementItem>,
}

impl<'a, D: Decoder + Copy> Iterator for DecodeElement<'a, D> {
    type Item = crate::Result<Action<D::Output<'a>>>;

    fn next(&mut self) -> Option<Self::Item> {
        let item: &'a ElementItem = self.items.next()?;
        let scanner = item.arguments.scan(self.decoder);
        Some(Action::new(item.atom.action, scanner))
    }
}
