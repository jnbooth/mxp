use std::borrow::Cow;

use super::line_tags::{LineTagUpdate, LineTags};
use crate::argument::KeywordFilter;
use crate::argument::{Arguments, Decoder};
use crate::collections::CaseFoldMap;
use crate::element::{
    Action, CollectedDefinition, DecodeElement, DefinitionKind, Element, ElementCommand, Mode, Tag,
};
use crate::entity::{DecodedEntity, EntityEntry, EntityMap, PublishedIter};
use crate::parser::{Error, ErrorKind, Words};
use crate::validate;

/// A store of MXP state: elements, entities, and line tags.
#[derive(Clone, Debug, Default)]
pub struct State {
    elements: CaseFoldMap<'static, Element>,
    entities: EntityMap,
    line_tags: LineTags,
}

impl State {
    /// Constructs a new `State`.
    ///
    /// Unlike `State::default()`, this function populates the state with elements and entities
    /// defined by the MXP protocol specification, allocating memory in the process.
    pub fn populated() -> Self {
        let mut elements = CaseFoldMap::<Element>::new();
        elements.extend(Element::well_known());
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

    /// Alias for `self.entities().get(name)`.
    /// See [`EntityMap::get`].
    pub fn get_entity(&self, name: &str) -> Option<&str> {
        self.entities.get(name)
    }

    /// Alias for `self.entities_mut().insert(name, value)`.
    /// See [`EntityMap::insert`].
    pub fn insert_entity(&mut self, name: String, value: String) -> bool {
        self.entities.insert(name, value)
    }

    /// Alias for `self.entities().published()`.
    /// See [`EntityMap::published`].
    pub fn published_entities(&self) -> PublishedIter<'_> {
        self.entities.published()
    }

    /// Retrieves a tag or element by name. Returns an error if no tag or element is defined by
    /// that name, or if the name is not a valid MXP identifier.
    pub fn get_component(&self, name: &str) -> crate::Result<Component<'_>> {
        if let Some(tag) = Tag::well_known(name) {
            Ok(Component::Tag(tag))
        } else if let Some(custom) = self.elements.get(name) {
            Ok(Component::Element(custom))
        } else {
            validate(name, ErrorKind::InvalidElementName)?;
            Err(Error::new(name, ErrorKind::UnknownElement))
        }
    }

    /// Retrieves the element associated with a line tag for a specified mode, if one exists.
    pub fn get_line_tag(&self, mode: Mode) -> Option<&Element> {
        self.line_tags.get(usize::from(mode.0), &self.elements)
    }

    /// Returns the number of custom MXP elements that have been stored.
    pub fn custom_elements_len(&self) -> usize {
        self.elements.len()
    }

    /// Returns the number of custom MXP entities that have been stored.
    /// Alias for `self.entities().len()`.
    /// See [`EntityMap::len`].
    pub fn custom_entities_len(&self) -> usize {
        self.entities.len()
    }

    /// Decodes the actions of an element, using the specified arguments.
    pub fn decode_element<'a>(
        &'a self,
        element: &'a Element,
        args: &'a Arguments<'a>,
    ) -> DecodeElement<'a, &'a State> {
        element.decode(args, self)
    }

    /// Decodes the value of an entity.
    /// Alias for `self.entities().decode(name)`.
    /// See [`EntityMap::decode`].
    pub fn decode_entity(&self, name: &str) -> crate::Result<Option<DecodedEntity<'_>>> {
        self.entities.decode(name)
    }

    /// Decodes the action of a predefined tag.
    pub fn decode_tag<'a>(
        &self,
        tag: &Tag,
        args: &'a Arguments<'a>,
    ) -> crate::Result<Action<Cow<'a, str>>> {
        tag.decode(args, self)
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
        let args = words.parse_args()?;
        let mut scanner = args.scan(&self.entities).with_keywords();
        let Some(value) = scanner.next()? else {
            return Err(Error::new(definition, ErrorKind::NoDefinitionTag));
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
            .extend::<String>(words)
    }
}

impl Decoder for State {
    fn decode_entity<F: KeywordFilter>(
        &self,
        entity: &str,
    ) -> crate::Result<Option<DecodedEntity<'_>>> {
        self.decode_entity(entity)
    }
}

/// This struct is created by [`State::get_component`]. See its documentation for more.
#[derive(Copy, Clone, Debug)]
pub enum Component<'a> {
    /// A user-defined custom tag element.
    Element(&'a Element),
    /// A built-in MXP tag.
    Tag(&'static Tag),
}

impl Component<'_> {
    /// Returns the name of the component.
    ///
    /// For example, the name of `<SOUND "ouch.wav">` is `"SOUND"`.
    pub const fn name(&self) -> &str {
        match self {
            Self::Element(el) => el.name.as_str(),
            Self::Tag(tag) => tag.name,
        }
    }

    /// Returns `true` if the element has no closing tag, e.g. `<BR>`.
    pub const fn is_command(&self) -> bool {
        match self {
            Self::Element(el) => el.command,
            Self::Tag(tag) => tag.action.is_command(),
        }
    }

    /// Returns `true` if the element is in Open mode, meaning users can override it.
    pub const fn is_open(&self) -> bool {
        match self {
            Self::Element(el) => el.open,
            Self::Tag(tag) => tag.action.is_open(),
        }
    }

    /// Returns the element's variable name, if it has one.
    pub const fn variable(&self) -> Option<&str> {
        match self {
            Self::Element(el) => match &el.variable {
                Some(name) => Some(name.as_str()),
                None => None,
            },
            Self::Tag(_) => None,
        }
    }
}
