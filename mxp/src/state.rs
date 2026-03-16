use std::borrow::Cow;

use crate::CaseFoldMap;
use crate::arguments::Arguments;
use crate::element::{Action, AtomicTag, Element, ElementDecoder};
use crate::elements::Var;
use crate::entity::{DecodedEntity, EntityEntry, EntityMap, PublishedIter};
use crate::keyword::KeywordFilter;
use crate::line::{LineTag, LineTags, Mode};
use crate::parse::{Decoder, Words};
use crate::parsed::{
    AttributeListDefinition, ParsedDefinition, ParsedElementDefinition, ParsedEntityDefinition,
    ParsedLineTagDefinition,
};
use crate::{Error, ErrorKind};

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
    pub fn with_globals() -> Self {
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

    /// Alias for `self.entities().guard_global(name)`.
    /// See [`EntityMap::guard_global`]
    pub fn guard_global_entity(&self, name: &str) -> crate::Result<()> {
        self.entities.guard_global(name)
    }

    /// Alias for `self.entities().is_global(name)`.
    /// See [`EntityMap::is_global`].
    pub fn is_global_entity(&self, name: &str) -> bool {
        self.entities.is_global(name)
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

    pub fn set_entity<'a, S: AsRef<str>>(
        &'a mut self,
        var: &Var<S>,
        value: &str,
    ) -> crate::Result<Option<EntityEntry<'a>>> {
        let entity = self.entities.define(var.with_value(value))?;
        Ok(EntityEntry::new(entity))
    }

    /// Alias for `self.entities().published()`.
    /// See [`EntityMap::published`].
    pub fn published_entities(&self) -> PublishedIter<'_> {
        self.entities.published()
    }

    /// Retrieves a tag or element by name. Returns an error if no tag or element is defined by
    /// that name, or if the tag or element is not Open  (see [`Component::is_open`]) and `secure`
    /// is false.
    pub fn get_component(&self, name: &str, secure: bool) -> crate::Result<Component<'_>> {
        let component = if let Some(custom) = self.elements.get(name) {
            Component::Element(custom)
        } else if let Some(tag) = AtomicTag::well_known(name) {
            Component::AtomicTag(tag)
        } else {
            return Err(Error::new(name, ErrorKind::UnknownElement));
        };
        if !secure && !component.is_open() {
            return Err(Error::new(name, ErrorKind::UnsecuredElement));
        }
        Ok(component)
    }

    /// Retrieves the element associated with a line tag for a specified mode, if one exists.
    pub fn get_line_tag(&self, mode: Mode) -> Option<LineTag<'_>> {
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
    ) -> ElementDecoder<'a, &'a State> {
        element.decode(args, self)
    }

    /// Decodes the value of an entity.
    /// Alias for `self.entities().decode(name)`.
    /// See [`EntityMap::decode`].
    pub fn decode_entity(&self, name: &str) -> crate::Result<DecodedEntity<'_>> {
        self.entities.decode(name)
    }

    /// Decodes the action of a predefined tag.
    pub fn decode_tag<'a>(
        &self,
        tag: &AtomicTag,
        args: &'a Arguments<'a>,
    ) -> crate::Result<Action<Cow<'a, str>>> {
        tag.decode(args, self)
    }

    /// Handles an MXP definition from the server, which may define an [attribute list], [element],
    /// [entity], or [line tag].
    ///
    /// Returns an [`EntityEntry`] if the operation alters the definition of an entity. The client
    /// can use this to keep track of entity updates, especially if the entity has
    /// [`EntityVisibility::Publish`].
    ///
    /// [attribute list]: https://www.zuggsoft.com/zmud/mxp.htm#ATTLIST
    /// [element]: https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT
    /// [entity]: https://www.zuggsoft.com/zmud/mxp.htm#ENTITY
    /// [line tag]: https://www.zuggsoft.com/zmud/mxp.htm#User-defined%20Line%20Tags
    /// [`EntityVisibility::Publish`]: crate::entity::EntityVisibility::Publish
    pub fn define<'a>(
        &'a mut self,
        definition: ParsedDefinition,
    ) -> crate::Result<Option<EntityEntry<'a>>> {
        match definition {
            ParsedDefinition::AttributeList(def) => self.define_attributes(&def)?,
            ParsedDefinition::Element(def) => self.define_element(def),
            ParsedDefinition::Entity(def) => return self.define_entity(def),
            ParsedDefinition::LineTag(def) => self.define_line_tag(def)?,
        }
        Ok(None)
    }

    fn define_attributes(&mut self, definition: &AttributeListDefinition) -> crate::Result<()> {
        let words = Words::new(definition.attributes);
        self.elements
            .get_mut(definition.name)
            .ok_or_else(|| Error::new(definition.name, ErrorKind::UnknownElementInAttlist))?
            .attributes
            .extend(words)
    }

    fn define_element(&mut self, definition: ParsedElementDefinition) {
        let Some(el) = definition.element else {
            self.elements.remove(definition.name);
            return;
        };
        if let Some(tag) = el.line_tag {
            self.line_tags.set(tag.0.into(), el.name.clone());
        }
        self.elements.insert(el.name.clone(), el);
    }

    fn define_entity<'a>(
        &'a mut self,
        definition: ParsedEntityDefinition,
    ) -> crate::Result<Option<EntityEntry<'a>>> {
        let ParsedEntityDefinition {
            name,
            desc,
            value,
            keywords,
        } = definition;
        let desc = match desc {
            Some(desc) => Some(self.decode_string::<()>(desc)?),
            None => None,
        };
        let value = self.decode_string::<()>(value)?;
        let entity = self.entities.define(ParsedEntityDefinition {
            name,
            desc: desc.as_deref(),
            value: &value,
            keywords,
        })?;
        Ok(EntityEntry::new(entity))
    }

    fn define_line_tag(&mut self, definition: ParsedLineTagDefinition) -> crate::Result<()> {
        self.line_tags.update(definition)
    }
}

impl Decoder for State {
    fn get_entity<K: KeywordFilter>(&self, name: &str) -> Option<&str> {
        self.entities.get_entity::<K>(name)
    }
}

/// This struct is created by [`State::get_component`]. See its documentation for more.
#[derive(Copy, Clone, Debug)]
pub enum Component<'a> {
    /// A built-in MXP tag.
    AtomicTag(&'static AtomicTag),
    /// A user-defined custom tag element.
    Element(&'a Element),
}

impl Component<'_> {
    /// Returns the name of the component.
    ///
    /// For example, the name of `<SOUND "ouch.wav">` is `"SOUND"`.
    pub const fn name(&self) -> &str {
        match self {
            Self::AtomicTag(tag) => tag.name,
            Self::Element(el) => el.name.as_str(),
        }
    }

    /// Returns `true` if the element has no closing tag, e.g. `<BR>`.
    pub const fn is_command(&self) -> bool {
        match self {
            Self::AtomicTag(tag) => tag.action.is_command(),
            Self::Element(el) => el.command,
        }
    }

    /// Returns `true` if the element is in Open mode, meaning users can override it.
    pub const fn is_open(&self) -> bool {
        match self {
            Self::AtomicTag(tag) => tag.action.is_open(),
            Self::Element(el) => el.open,
        }
    }

    /// Returns the element's variable name, if it has one.
    pub const fn variable(&self) -> Option<&str> {
        match self {
            Self::AtomicTag(_) => None,
            Self::Element(el) => match &el.variable {
                Some(name) => Some(name.as_str()),
                None => None,
            },
        }
    }
}
