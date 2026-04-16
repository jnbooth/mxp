use std::borrow::Cow;

use crate::CaseFoldMap;
use crate::arguments::Arguments;
use crate::element::{Action, AtomicTag, Element, ElementFlag};
use crate::elements::Var;
use crate::entity::{DecodedEntity, EntityEntry, EntityMap, PublishedIter};
use crate::line::{LineTag, LineTags, Mode};
use crate::node::{
    AttributeListDefinition, Definition, ElementDefinition, EntityDefinition, LineTagDefinition,
};
use crate::parse::Decoder;
use crate::{Error, ErrorKind};

/// A store of MXP state: elements, entities, and line tags.
#[derive(Debug, Default)]
pub struct State {
    elements: CaseFoldMap<'static, Element>,
    entities: EntityMap,
    line_tags: LineTags,
}

impl Clone for State {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            elements: self.elements.clone(),
            entities: self.entities.clone(),
            line_tags: self.line_tags.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.elements.clone_from(&source.elements);
        self.entities.clone_from(&source.entities);
        self.line_tags.clone_from(&source.line_tags);
    }
}

impl State {
    /// Constructs a new `State`.
    ///
    /// Unlike `State::default()`, this function populates the state with elements and entities
    /// defined by the MXP protocol specification, allocating memory in the process.
    ///
    /// # Examples
    ///
    /// ```
    /// let state = mxp::State::with_globals();
    /// ```
    pub fn with_globals() -> Self {
        let mut elements = CaseFoldMap::new();
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
    /// See [`EntityMap::guard_global`].
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

    /// Applies a [`<VAR>`] action, using the specified `value` which is the text that was sent by
    /// the server in between the opening and closing tag (e.g. `<VAR Hp>value</VAR>`). Note that
    /// if [`var.keywords`] contains [`EntityKeyword::Delete`], or if it contains
    /// [`EntityKeyword::Remove`] and `value` was the only value in the entity's list, this will set
    /// the entity to `None`.
    ///
    /// Returns an error if the name is associated with a global XML entity, since those cannot be
    /// changed. Returns `None` if the entity's [`visibility`] is [`EntityVisibility::Private`],
    /// because private entities are hidden from the client. Otherwise, returns an `EntityEntry`
    /// whose [`value`] is `Some` if the entity was inserted or updated, and `None` if it was
    /// removed. As with [`define`], the client can use this to keep track of entity updates,
    /// especially if the entity has [`EntityVisibility::Publish`].
    ///
    /// [`<VAR>`]: Var
    /// [`var.keywords`]: [`Var::keywords`]
    /// [`EntityKeyword::Delete`]: crate::keyword::EntityKeyword::Delete
    /// [`EntityKeyword::Remove`]: crate::keyword::EntityKeyword::Remove
    /// [`visibility`]: crate::entity::Entity::visibility
    /// [`EntityVisibility::Private`]: crate::entity::EntityVisibility::Private
    /// [`value`]: EntityEntry::value
    /// [`define`]: Self::define
    /// [`EntityVisibility::Publish`]: crate::entity::EntityVisibility::Publish
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
    /// that name, or if the tag or element is not OPEN (see [`Component::is_open`]) and `secure`
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
        definition: Definition,
    ) -> crate::Result<Option<EntityEntry<'a>>> {
        match definition {
            Definition::AttributeList(def) => self.define_attributes(&def)?,
            Definition::Element(def) => self.define_element(def),
            Definition::Entity(def) => return self.define_entity(def),
            Definition::LineTag(def) => self.define_line_tag(def)?,
        }
        Ok(None)
    }

    fn define_attributes(&mut self, definition: &AttributeListDefinition) -> crate::Result<()> {
        let attributes = &mut self
            .elements
            .get_mut(definition.name)
            .ok_or_else(|| Error::new(definition.name, ErrorKind::UnknownElementInAttlist))?
            .attributes;
        let len = attributes.len();
        let result = attributes.append(definition.attributes);
        if result.is_err() {
            attributes.truncate(len);
        }
        result
    }

    fn define_element(&mut self, definition: ElementDefinition) {
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
        definition: EntityDefinition,
    ) -> crate::Result<Option<EntityEntry<'a>>> {
        let EntityDefinition {
            name,
            desc,
            value,
            keywords,
        } = definition;
        let desc = match desc {
            Some(desc) => Some(self.decode_string(desc)?),
            None => None,
        };
        let value = self.decode_string(value)?;
        let entity = self.entities.define(EntityDefinition {
            name,
            desc: desc.as_deref(),
            value: &value,
            keywords,
        })?;
        Ok(EntityEntry::new(entity))
    }

    fn define_line_tag(&mut self, definition: LineTagDefinition) -> crate::Result<()> {
        self.line_tags.update(definition)
    }
}

impl Decoder for State {
    fn get_entity(&self, name: &str) -> Option<&str> {
        self.entities.get_entity(name)
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
            Self::Element(el) => el.empty,
        }
    }

    /// Returns `true` if the element is in OPEN mode, meaning users can override it.
    pub const fn is_open(&self) -> bool {
        match self {
            Self::AtomicTag(tag) => tag.action.is_open(),
            Self::Element(el) => el.open,
        }
    }

    /// Returns the element's flag, if it has one.
    pub const fn flag(&self) -> Option<&ElementFlag> {
        match self {
            Self::AtomicTag(_) => None,
            Self::Element(el) => el.flag.as_ref(),
        }
    }
}
