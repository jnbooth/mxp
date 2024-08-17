use std::borrow::Cow;
use std::slice;

use super::element_map::{ElementComponent, ElementMap};
use super::line_tags::{LineTagUpdate, LineTags};
use crate::argument::{Arguments, Decoder, ElementDecoder, Scan};
use crate::element::{Action, Element, ElementItem, Mode};
use crate::entity::{EntityEntry, EntityMap, PublishedIter};
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
        self.line_tags.clear();
    }

    pub fn add_globals(&mut self) {
        self.entities.add_globals();
    }

    pub fn is_global_entity(&self, key: &str) -> bool {
        self.entities.is_global(key)
    }

    pub fn entities_mut(&mut self) -> &mut EntityMap {
        &mut self.entities
    }

    pub fn published_entities(&self) -> PublishedIter {
        self.entities.published()
    }

    pub fn get_component(&self, name: &str) -> crate::Result<ElementComponent> {
        self.elements.get_component(name)
    }

    pub fn get_entity(&self, name: &str) -> crate::Result<Option<&str>> {
        self.entities.decode_entity(name)
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
        DecodeElement {
            items: element.items.iter(),
            decoder: ElementDecoder {
                element,
                args,
                entities: &self.entities,
            },
        }
    }

    pub fn define<'a>(&'a mut self, tag: &'a str) -> crate::Result<Option<EntityEntry<'a>>> {
        let mut words = Words::new(tag);

        let definition = words.validate_next_or(ErrorKind::InvalidDefinition)?;
        if definition.eq_ignore_ascii_case("tag") {
            self.define_line_tag(words)?;
            return Ok(None);
        }
        let name = words.validate_next_or(ErrorKind::InvalidElementName)?;
        match_ci! {definition,
            "ATTLIST" | "ATT" => self.define_attributes(name, words),
            "ELEMENT" | "EL" => self.define_element(name, words),
            "ENTITY" | "EN" => return self.define_entity(name, words),
            _ => Err(Error::new(definition, ErrorKind::InvalidDefinition))
        }?;
        Ok(None)
    }

    fn define_element(&mut self, name: &str, words: Words) -> crate::Result<()> {
        let args = Arguments::parse(words)?;
        let Some(el) = Element::parse(name.to_owned(), args.scan(&self.entities))? else {
            self.elements.remove(&name);
            return Ok(());
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

    fn define_entity<'a>(
        &'a mut self,
        key: &'a str,
        words: Words,
    ) -> crate::Result<Option<EntityEntry<'a>>> {
        if self.entities.is_global(key) {
            return Err(Error::new(key, ErrorKind::CannotRedefineEntity));
        }
        let s = words.as_str();
        let args = Arguments::parse(words)?;
        let mut scanner = args.scan(&self.entities).with_keywords();
        let Some(value) = scanner.next()? else {
            return Err(Error::new(s, ErrorKind::NoDefinitionTag));
        };
        let desc = scanner.next_or("desc")?;
        let keywords = scanner.into_keywords();
        let entity = self
            .entities
            .set(key, &value, desc.map(Cow::into_owned), keywords);
        Ok(entity)
    }

    fn define_attributes(&mut self, key: &str, words: Words) -> crate::Result<()> {
        self.elements
            .get_mut(key)
            .ok_or_else(|| Error::new(key, ErrorKind::UnknownElementInAttlist))?
            .attributes
            .append(words)
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
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
