use std::ops::{Deref, DerefMut};

use casefold::ascii::{CaseFold, CaseFoldMap};

use crate::element::{Element, Tag, Tags};
use crate::parser::{Error, ErrorKind, validate};

/// A component in an [element definition](https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT).
#[derive(Copy, Clone, Debug)]
pub enum ElementComponent<'a> {
    /// A user-defined custom tag element.
    Element(&'a Element),
    /// A built-in MXP tag.
    Tag(&'static Tag),
}

impl ElementComponent<'_> {
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ElementMap {
    inner: CaseFoldMap<String, Element>,
}

impl Deref for ElementMap {
    type Target = CaseFoldMap<String, Element>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ElementMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ElementMap {
    pub fn well_known() -> Self {
        Self {
            inner: well_known_elements(),
        }
    }

    pub fn get_component(&self, key: &str, tags: &Tags) -> crate::Result<ElementComponent<'_>> {
        validate(key, ErrorKind::InvalidElementName)?;

        if let Some(tag) = tags.get(key) {
            Ok(ElementComponent::Tag(tag))
        } else if let Some(custom) = self.get(key) {
            Ok(ElementComponent::Element(custom))
        } else {
            Err(Error::new(key, ErrorKind::UnknownElement))
        }
    }
}

fn well_known_elements() -> CaseFoldMap<String, Element> {
    fn color_el(name: &'static str, hex: u32) -> (CaseFold<String>, Element) {
        (
            name.to_owned().into(),
            Element {
                name: name.to_owned(),
                open: true,
                fore: hex.try_into().ok(),
                ..Default::default()
            },
        )
    }

    [
        color_el("BlackMXP", 0x000000),
        color_el("RedMXP", 0xFF0000),
        color_el("GreenMXP", 0x008000),
        color_el("YellowMXP", 0xFFFF00),
        color_el("BlueMXP", 0x0000FF),
        color_el("MagentaMXP", 0xFF00FF),
        color_el("CyanMXP", 0x00FFFF),
        color_el("WhiteMXP", 0xFFFFFF),
    ]
    .into_iter()
    .collect()
}
