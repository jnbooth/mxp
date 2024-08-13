use std::ops::{Deref, DerefMut};

use casefold::ascii::CaseFoldMap;
use enumeration::EnumSet;

use crate::lookup::Lookup;

use crate::argument::Arguments;
use crate::entity::{Atom, Element, ElementItem, TagFlag};
use crate::parser::{validate, Error as MxpError, ParseError};

#[derive(Copy, Clone, Debug)]
pub enum ElementComponent<'a> {
    Atom(&'static Atom),
    Custom(&'a Element),
}

impl<'a> ElementComponent<'a> {
    pub fn name(&self) -> &str {
        match self {
            Self::Atom(atom) => atom.name.as_str(),
            Self::Custom(el) => el.name.as_str(),
        }
    }

    pub const fn flags(&self) -> EnumSet<TagFlag> {
        match self {
            Self::Atom(atom) => atom.flags,
            Self::Custom(el) => el.flags(),
        }
    }

    pub fn variable(&self) -> Option<String> {
        match self {
            Self::Atom(_) => None,
            Self::Custom(el) => el.variable.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ElementMap(CaseFoldMap<String, Element>);

impl Deref for ElementMap {
    type Target = CaseFoldMap<String, Element>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ElementMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ElementMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_component(&self, key: &str) -> Result<ElementComponent, ParseError> {
        validate(key, MxpError::InvalidElementName)?;

        if let Some(atom) = Atom::get(key) {
            Ok(ElementComponent::Atom(atom))
        } else if let Some(custom) = self.get(key) {
            Ok(ElementComponent::Custom(custom))
        } else if let Some(custom) = WELL_KNOWN_ELEMENTS.get(key) {
            Ok(ElementComponent::Custom(custom))
        } else {
            Err(ParseError::new(key, MxpError::UnknownElement))
        }
    }
}

static WELL_KNOWN_ELEMENTS: Lookup<Element> = Lookup::new(|| {
    let color_atom = Atom::get("color").unwrap();
    let color_el = |color: &'static str| {
        let mut arguments = Arguments::new();
        arguments.push(color[..color.len() - "MXP".len()].to_ascii_lowercase());
        let el = Element {
            name: color.to_owned(),
            items: vec![ElementItem {
                atom: color_atom,
                arguments,
            }],
            attributes: Arguments::new(),
            tag: None,
            variable: None,
            open: true,
            command: false,
        };
        (color, el)
    };
    vec![
        color_el("BlackMXP"),
        color_el("RedMXP"),
        color_el("GreenMXP"),
        color_el("YellowMXP"),
        color_el("BlueMXP"),
        color_el("MagentaMXP"),
        color_el("CyanMXP"),
        color_el("WhiteMXP"),
    ]
});
