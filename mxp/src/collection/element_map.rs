use std::ops::{Deref, DerefMut};

use casefold::ascii::CaseFoldMap;

use crate::lookup::Lookup;

use crate::argument::Arguments;
use crate::color::RgbColor;
use crate::element::{Element, Tag};
use crate::parser::{validate, Error, ErrorKind};

#[derive(Copy, Clone, Debug)]
pub enum ElementComponent<'a> {
    Element(&'a Element),
    Tag(&'static Tag),
}

impl<'a> ElementComponent<'a> {
    pub fn name(&self) -> &str {
        match self {
            Self::Element(el) => el.name.as_str(),
            Self::Tag(tag) => tag.name.as_str(),
        }
    }

    pub const fn is_command(&self) -> bool {
        match self {
            Self::Element(el) => el.command,
            Self::Tag(tag) => tag.command,
        }
    }

    pub const fn is_open(&self) -> bool {
        match self {
            Self::Element(el) => el.open,
            Self::Tag(tag) => tag.open,
        }
    }

    pub fn variable(&self) -> Option<String> {
        match self {
            Self::Element(el) => el.variable.clone(),
            Self::Tag(_) => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ElementMap(CaseFoldMap<String, Element>);

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
    pub fn get_component(&self, key: &str) -> crate::Result<ElementComponent> {
        validate(key, ErrorKind::InvalidElementName)?;

        if let Some(tag) = Tag::get(key) {
            Ok(ElementComponent::Tag(tag))
        } else if let Some(custom) = self.get(key) {
            Ok(ElementComponent::Element(custom))
        } else if let Some(custom) = WELL_KNOWN_ELEMENTS.get(key) {
            Ok(ElementComponent::Element(custom))
        } else {
            Err(Error::new(key, ErrorKind::UnknownElement))
        }
    }
}

static WELL_KNOWN_ELEMENTS: Lookup<Element> = Lookup::new(|| {
    let color_el = |name: &'static str, hex: u32| {
        let el = Element {
            name: name.to_owned(),
            attributes: Arguments::new(),
            items: Vec::new(),
            tag: None,
            parse_as: None,
            variable: None,
            open: true,
            command: false,
            fore: Some(RgbColor::hex(hex)),
            back: None,
            gag: false,
            window: None,
        };
        (name, el)
    };
    vec![
        color_el("BlackMXP", 0x000000),
        color_el("RedMXP", 0xFF0000),
        color_el("GreenMXP", 0x008000),
        color_el("YellowMXP", 0xFFFF00),
        color_el("BlueMXP", 0x0000FF),
        color_el("MagentaMXP", 0xFF00FF),
        color_el("CyanMXP", 0x00FFFF),
        color_el("WhiteMXP", 0xFFFFFF),
    ]
});
