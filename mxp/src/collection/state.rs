use std::slice;

use super::element_map::{ElementComponent, ElementMap};
use super::entity_map::{ElementDecoder, EntityMap};
use super::line_tags::{LineTagUpdate, LineTags};
use super::published_entities::{PublishedEntities, PublishedEntity};
use crate::argument::scan::{Decoder, Scan};
use crate::argument::Arguments;
use crate::entity::{Action, Element, ElementItem, Mode};
use crate::keyword::EntityKeyword;
use crate::parser::{Error, ErrorKind, Words};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    elements: ElementMap,
    entities: EntityMap,
    line_tags: LineTags,
    published: PublishedEntities,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.entities.clear();
        self.line_tags.clear();
        self.published.clear();
    }

    pub fn published_entities(&self) -> slice::Iter<PublishedEntity> {
        self.published.iter()
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
        let el = match Element::parse(name.to_owned(), args.scan(&self.entities))? {
            Some(el) => el,
            None => {
                self.elements.remove(&name);
                return Ok(());
            }
        };
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

    fn define_entity(&mut self, key: &str, words: Words) -> crate::Result<()> {
        if EntityMap::global(key).is_some() {
            return Err(Error::new(key, ErrorKind::CannotRedefineEntity));
        }
        let s = words.as_str();
        let args = Arguments::parse(words)?;
        let mut scanner = args.scan(&self.entities).with_keywords();
        let value = match scanner.next()? {
            Some(value) => value,
            None => return Err(Error::new(s, ErrorKind::NoDefinitionTag)),
        };
        let desc = scanner.next_or(&["desc"])?;
        let keywords = scanner.into_keywords();
        if keywords.contains(EntityKeyword::Delete) {
            self.entities.remove(key);
            self.published.remove(key);
            return Ok(());
        }
        if keywords.contains(EntityKeyword::Private) {
            self.published.remove(key);
        } else if keywords.contains(EntityKeyword::Publish) {
            let desc = match desc {
                Some(desc) => desc.into_owned(),
                None => String::new(),
            };
            self.published.insert(key.to_owned(), desc)
        }
        if keywords.contains(EntityKeyword::Remove) {
            self.entities.remove_list_item(key, &value);
            return Ok(());
        }
        if keywords.contains(EntityKeyword::Add) {
            self.entities.add_list_item(key, &value);
            return Ok(());
        }
        self.entities.insert(key.to_owned(), value.into_owned());
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
