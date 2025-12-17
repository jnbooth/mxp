use std::borrow::Cow;
use std::slice;

use flagset::FlagSet;

use super::element_map::{ElementComponent, ElementMap};
use super::line_tags::{LineTagUpdate, LineTags};
use crate::argument::{Arguments, Decoder, ElementDecoder};
use crate::element::{
    Action, ActionKind, CollectedDefinition, DefinitionKind, Element, ElementItem, Mode, Tag, Tags,
};
use crate::entity::{EntityEntry, EntityMap, PublishedIter};
use crate::parser::{Error, ErrorKind, Words};
use crate::responses::SupportResponse;

/// A store of MXP state: elements, entities, and line tags.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    elements: ElementMap,
    entities: EntityMap,
    line_tags: LineTags,
    tags: Tags,
}

impl State {
    /// Constructs a new `State`.
    ///
    /// Unlike `State::default()`, this function populates the state with elements and entities
    /// defined by the MXP protocol specification, allocating memory in the process.
    pub fn populated() -> Self {
        Self {
            elements: ElementMap::well_known(),
            entities: EntityMap::with_globals(),
            line_tags: LineTags::new(),
            tags: Tags::well_known(),
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
    pub fn is_global_entity(&self, key: &str) -> bool {
        self.entities.is_global(key)
    }

    /// Mutably borrows the map of defined MXP entities.
    pub fn entities_mut(&mut self) -> &mut EntityMap {
        &mut self.entities
    }

    /// Iterates through published entities.
    pub fn published_entities(&self) -> PublishedIter<'_> {
        self.entities.published()
    }

    /// Retrieves a tag or element by name.
    pub fn get_component(&self, name: &str) -> crate::Result<ElementComponent<'_>> {
        self.elements.get_component(name, &self.tags)
    }

    /// Retrieves the element associated with a line tag for a specified mode, if one exists.
    pub fn get_line_tag(&self, mode: Mode) -> Option<&Element> {
        self.line_tags.get(usize::from(mode.0), &self.elements)
    }

    /// Creates a formatting `struct` that outputs a [`<SUPPORT>`] response.
    ///
    /// [`<SUPPORT>`]: https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control
    pub fn supported_tags<I>(
        &self,
        iter: I,
        supported: FlagSet<ActionKind>,
    ) -> SupportResponse<'_, I>
    where
        I: IntoIterator + Copy,
        I::Item: AsRef<str>,
    {
        SupportResponse::new(iter, supported, &self.tags)
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
    pub fn decode_entity(&self, name: &str) -> crate::Result<Option<&str>> {
        self.entities.decode_entity(name)
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
        let mut words = Words::new(definition);
        let name = words.validate_next_or(ErrorKind::InvalidElementName)?;
        let args = words.parse_args::<String>()?;
        let Some(el) = Element::parse(name.to_owned(), args.scan(&self.entities), &self.tags)?
        else {
            self.elements.remove(&name);
            return Ok(());
        };
        if let Some(tag) = el.tag {
            self.line_tags.set(usize::from(tag.get()), el.name.clone());
        }
        self.elements.insert(name.to_owned(), el);
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

impl<'a, D: Decoder + Copy> Iterator for DecodeElement<'a, D> {
    type Item = crate::Result<Action<Cow<'a, str>>>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items.next()?;
        let scanner = item.arguments.scan(self.decoder);
        Some(Action::parse(item.tag.action, scanner))
    }
}
