use crate::Action;

use super::argument::{ArgumentIndex, Arguments, Keyword};
use super::element::{Element, ElementComponent, ElementMap};
use super::entity_map::EntityMap;
use super::error::{Error as MxpError, ParseError};
use super::words::Words;

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

    pub fn decode_args(&self, args: &mut Arguments) -> Result<(), ParseError> {
        for value in args.values_mut() {
            *value = self.entities.decode(value)?;
        }
        Ok(())
    }

    pub fn decode_element<'a>(
        &'a self,
        element: &'a Element,
        args: &'a Arguments,
    ) -> impl Iterator<Item = Result<(Action, Arguments), ParseError>> + 'a {
        element.items.iter().map(move |item| {
            let mut newargs = Arguments::new();
            for (i, arg) in &item.arguments {
                let val = self.entities.decode_el(element, arg, args)?;
                match i {
                    ArgumentIndex::Positional(_) => newargs.push(val),
                    ArgumentIndex::Named(key) => newargs.set(key, val),
                }
            }
            Ok((item.atom.action, newargs))
        })
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
        let args = Arguments::parse_words(words)?;
        if args.has_keyword(Keyword::Delete) {
            self.elements.remove(&name);
            return Ok(());
        }
        let el = Element::parse(name.to_owned(), args)?;
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
                let value = self.entities.decode(body)?;
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
