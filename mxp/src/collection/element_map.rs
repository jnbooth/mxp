use std::ops::{Deref, DerefMut};

use casefold::ascii::CaseFoldMap;

use crate::lookup::Lookup;

use crate::argument::Arguments;
use crate::color::RgbColor;
use crate::element::{Element, Tag};
use crate::parser::{validate, Error, ErrorKind};

/// A component in an [element definition](https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT).
#[derive(Copy, Clone, Debug)]
pub enum ElementComponent<'a> {
    /// A user-defined custom tag element.
    Element(&'a Element),
    /// A built-in MXP tag.
    Tag(&'static Tag),
}

impl<'a> ElementComponent<'a> {
    /// Returns the name of the component.
    ///
    /// For example, the name of `<SOUND "ouch.wav">` is `"SOUND"`.
    pub fn name(&self) -> &str {
        match self {
            Self::Element(el) => &el.name,
            Self::Tag(tag) => &tag.name,
        }
    }

    /// Returns `true` if the element has no closing tag, e.g. `<BR>`.
    pub fn is_command(&self) -> bool {
        match self {
            Self::Element(el) => el.command,
            Self::Tag(tag) => tag.command,
        }
    }

    /// Returns `true` if the element is in Open mode, meaning users can override it.
    pub fn is_open(&self) -> bool {
        match self {
            Self::Element(el) => el.open,
            Self::Tag(tag) => tag.open,
        }
    }

    /// Returns the element's variable name, if it has one.
    pub fn variable(&self) -> Option<&str> {
        match self {
            Self::Element(el) => el.variable.as_deref(),
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
