use std::borrow::Cow;
use std::iter::FusedIterator;
use std::slice;

use super::element_map::{ElementComponent, ElementMap};
use super::line_tags::{LineTagUpdate, LineTags};
use crate::argument::{Arguments, Decoder, ElementDecoder};
use crate::element::{
    Action, CollectedDefinition, DefinitionKind, Element, ElementCommand, ElementItem, Mode, Tag,
};
use crate::entity::{DecodedEntity, EntityEntry, EntityMap};
use crate::parser::{Error, ErrorKind, Words};

/// A store of MXP state: elements, entities, and line tags.
#[derive(Clone, Debug, Default)]
pub struct State {
    elements: ElementMap,
    entities: EntityMap,
    line_tags: LineTags,
}

impl State {
    /// Constructs a new `State`.
    ///
    /// Unlike `State::default()`, this function populates the state with elements and entities
    /// defined by the MXP protocol specification, allocating memory in the process.
    pub fn populated() -> Self {
        let mut elements = ElementMap::new();
        elements.add_well_known();
        Self {
            elements,
            entities: EntityMap::with_globals(),
            line_tags: LineTags::new(),
        }
    }

    /// Clears the state, removing all elements, entities, and line tags, except for predefined
    /// globals.
    pub fn clear(&mut self) {
        self.elements.clear();
        self.entities.clear();
        self.line_tags.clear();
    }

    /// Returns `true` if the specified name belongs to a global entity as predefined by the MXP
    /// protocol specifications.
    ///
    /// # Examples
    ///
    /// ```
    /// let state = mxp::State::populated();
    /// assert!(state.is_global_entity("lt"));
    /// assert!(!state.is_global_entity("thomas"));
    /// ```
    pub fn is_global_entity(&self, key: &str) -> bool {
        self.entities.is_global(key)
    }

    /// Borrows the map of defined MXP entities.
    pub fn entities(&self) -> &EntityMap {
        &self.entities
    }

    /// Mutably borrows the map of defined MXP entities.
    pub fn entities_mut(&mut self) -> &mut EntityMap {
        &mut self.entities
    }

    /// Retrieves a tag or element by name. Returns an error if no tag or element is defined by
    /// that name, or if the name is not a valid MXP identifier.
    pub fn get_component(&self, name: &str) -> crate::Result<ElementComponent<'_>> {
        self.elements.get_component(name)
    }

    /// Retrieves the element associated with a line tag for a specified mode, if one exists.
    pub fn get_line_tag(&self, mode: Mode) -> Option<&Element> {
        self.line_tags.get(usize::from(mode.0), &self.elements)
    }

    /// Returns the number of custom MXP elements that have been stored.
    pub fn count_custom_elements(&self) -> usize {
        self.elements.len()
    }

    /// Decodes the actions of an element, using the specified arguments.
    pub fn decode_element<'a, S: AsRef<str>>(
        &'a self,
        element: &'a Element,
        args: &'a Arguments<S>,
    ) -> DecodeElement<'a, ElementDecoder<'a, S>> {
        DecodeElement {
            items: element.items.iter(),
            decoder: ElementDecoder {
                element,
                args,
                entities: &self.entities,
            },
        }
    }

    /// Decodes the value of an entity.
    pub fn decode_entity(&self, name: &str) -> crate::Result<Option<DecodedEntity<'_>>> {
        self.entities.decode(name)
    }

    /// Decodes the action of a predefined tag.
    pub fn decode_tag<'a, S: AsRef<str>>(
        &self,
        tag: &Tag,
        args: &'a Arguments<S>,
    ) -> crate::Result<Action<Cow<'a, str>>> {
        Action::parse(tag.action, args.scan(&self.entities))
    }

    /// Handles an MXP definition from the server, which may define an [attribute list], [element],
    /// [entity], or [line tag].
    ///
    /// [attribute list]: https://www.zuggsoft.com/zmud/mxp.htm#ATTLIST
    /// [element]: https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT
    /// [entity]: https://www.zuggsoft.com/zmud/mxp.htm#ENTITY
    /// [line tag]: https://www.zuggsoft.com/zmud/mxp.htm#User-defined%20Line%20Tags
    pub fn define<'a>(
        &'a mut self,
        definition: CollectedDefinition<'a>,
    ) -> crate::Result<Option<EntityEntry<'a>>> {
        match definition.kind {
            DefinitionKind::AttributeList => self.define_attributes(definition.text),
            DefinitionKind::Element => self.define_element(definition.text),
            DefinitionKind::Entity => return self.define_entity(definition.text),
            DefinitionKind::LineTag => self.define_line_tag(definition.text),
        }?;
        Ok(None)
    }

    fn define_element(&mut self, definition: &str) -> crate::Result<()> {
        let el = match Element::parse(definition, &self.entities)? {
            ElementCommand::Define(el) => el,
            ElementCommand::Delete(name) => {
                self.elements.remove(&name);
                return Ok(());
            }
        };
        if let Some(tag) = el.tag {
            self.line_tags.set(usize::from(tag.get()), el.name.clone());
        }
        self.elements.insert(el.name.clone(), el);
        Ok(())
    }

    fn define_line_tag(&mut self, definition: &str) -> crate::Result<()> {
        let update = LineTagUpdate::parse(Words::new(definition), &self.entities)?;
        self.line_tags.update(update, &mut self.elements);
        Ok(())
    }

    fn define_entity<'a>(
        &'a mut self,
        definition: &'a str,
    ) -> crate::Result<Option<EntityEntry<'a>>> {
        let mut words = Words::new(definition);
        let key = words.validate_next_or(ErrorKind::InvalidElementName)?;
        let s = words.as_str();
        let args = words.parse_args::<&str>()?;
        let mut scanner = args.scan(&self.entities).with_keywords();
        let Some(value) = scanner.next()? else {
            return Err(Error::new(s, ErrorKind::NoDefinitionTag));
        };
        let desc = scanner.next_or("desc")?;
        let keywords = scanner.into_keywords();
        self.entities
            .set(key, &value, desc.map(Cow::into_owned), keywords)
    }

    fn define_attributes(&mut self, definition: &str) -> crate::Result<()> {
        let mut words = Words::new(definition);
        let key = words.validate_next_or(ErrorKind::InvalidElementName)?;
        self.elements
            .get_mut(key)
            .ok_or_else(|| Error::new(key, ErrorKind::UnknownElementInAttlist))?
            .attributes
            .append(words)
    }
}

/// This `struct` is created by [`State::decode_element`]. See its documentation for more.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct DecodeElement<'a, D> {
    decoder: D,
    items: slice::Iter<'a, ElementItem<String>>,
}

impl<'a, D> Iterator for DecodeElement<'a, D>
where
    D: Decoder + Copy,
{
    type Item = crate::Result<Action<Cow<'a, str>>>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items.next()?;
        let scanner = item.arguments.scan(self.decoder);
        Some(Action::parse(item.tag.action, scanner))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.len();
        (exact, Some(exact))
    }
}

impl<D> ExactSizeIterator for DecodeElement<'_, D>
where
    D: Decoder + Copy,
{
    fn len(&self) -> usize {
        self.items.len()
    }
}

impl<D> FusedIterator for DecodeElement<'_, D> where D: Decoder + Copy {}
