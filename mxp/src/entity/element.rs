use std::ops::{Deref, DerefMut};
use std::sync::OnceLock;

use casefold::ascii::{CaseFold, CaseFoldMap};
use enumeration::EnumSet;

use super::argument::{Arg, Arguments, Keyword};
use super::atom::{Atom, TagFlag};
use super::error::{Error as MxpError, ParseError};
use super::validation::validate;
use super::words::Words;

/// List of arguments to an MXP tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementItem {
    pub atom: &'static Atom,
    pub arguments: Arguments,
}

impl ElementItem {
    pub fn parse(tag: &str) -> Result<Self, ParseError> {
        let mut words = Words::new(tag);
        let atom_name = words
            .next()
            .ok_or_else(|| ParseError::new(tag, MxpError::NoDefinitionTag))?;
        let invalid_name = match atom_name {
            "/" => Some(MxpError::DefinitionCannotCloseElement),
            "!" => Some(MxpError::DefinitionCannotDefineElement),
            _ => None,
        };
        if let Some(invalid) = invalid_name {
            return Err(ParseError::new(words.next().unwrap_or(""), invalid));
        }
        let atom = Atom::get(atom_name)
            .ok_or_else(|| ParseError::new(atom_name, MxpError::NoInbuiltDefinitionTag))?;
        Ok(Self {
            atom,
            arguments: Arguments::parse_words(words)?,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CollectedElement<'a> {
    Definition(&'a str),
    TagClose(&'a str),
    TagOpen(&'a str),
}

impl<'a> CollectedElement<'a> {
    pub fn from_str(text: &'a str) -> Result<Self, ParseError> {
        let tag = *text
            .as_bytes()
            .first()
            .ok_or_else(|| ParseError::new("collected element", MxpError::EmptyElement))?;

        match tag {
            b'!' => Ok(Self::Definition(&text[1..])),
            b'/' => Ok(Self::TagClose(&text[1..])),
            _ => Ok(Self::TagOpen(&text)),
        }
    }
}

/// User-defined MXP tags that we recognise, e.g. <boldcolor>.
/// For example: <!ELEMENT boldtext '<COLOR &col;><B>' ATT='col=red'>
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Element {
    /// Tag name
    pub name: String,
    /// What atomic elements it defines (arg 1)
    pub items: Vec<ElementItem>,
    /// List of attributes to this element (ATT="xx")
    pub attributes: Arguments,
    /// Line tag number (20 - 99) (TAG=n)
    pub tag: Option<u8>,
    /// Which variable to set (SET x)
    pub variable: Option<String>,
    /// Whether the element is open (OPEN)
    pub open: bool,
    /// Whether the element has no closing tag (EMPTY)
    pub command: bool,
}

impl Element {
    pub const fn flags(&self) -> EnumSet<TagFlag> {
        if self.open && self.command {
            enums![TagFlag::Open, TagFlag::Command]
        } else if self.open {
            enums![TagFlag::Open]
        } else if self.command {
            enums![TagFlag::Command]
        } else {
            enums![]
        }
    }

    pub fn collect(text: &str) -> Result<CollectedElement, ParseError> {
        CollectedElement::from_str(text)
    }

    fn parse_items(argument: Option<&Arg>) -> Result<Vec<ElementItem>, ParseError> {
        let definitions = match argument {
            Some(definitions) => definitions,
            None => return Ok(Vec::new()),
        };

        let size_guess = definitions.bytes().filter(|&c| c == b'<').count();
        let mut items = Vec::with_capacity(size_guess);

        let mut iter = definitions.char_indices();
        while let Some((start, startc)) = iter.next() {
            if startc != '<' {
                return Err(ParseError::new(definitions, MxpError::NoTagInDefinition));
            }
            loop {
                let (end, endc) = iter.next().ok_or_else(|| {
                    ParseError::new(definitions, MxpError::NoClosingDefinitionQuote)
                })?;
                if endc == '>' {
                    let definition = &definitions[start + 1..end];
                    items.push(ElementItem::parse(definition)?);
                    break;
                }
                if (endc == '\'' || endc == '"') && !iter.any(|(_, c)| c == endc) {
                    return Err(ParseError::new(
                        definitions,
                        MxpError::NoClosingDefinitionQuote,
                    ));
                }
            }
        }

        Ok(items)
    }

    pub fn parse(name: String, arguments: Arguments) -> Result<Self, ParseError> {
        let mut scanner = arguments.scan();
        let items = Self::parse_items(scanner.next())?;

        let attributes = match scanner.next_or(&["att"]) {
            Some(atts) => Arguments::parse(atts)?,
            None => Arguments::default(),
        };

        let tag = match scanner.next_or(&["tag"]).and_then(|s| s.parse().ok()) {
            Some(i) if !(20..=99).contains(&i) => None,
            tag => tag,
        };

        let flag = scanner.next_or(&["flag"]).map(|flag| {
            flag.strip_prefix("set ")
                .unwrap_or(flag)
                .trim()
                .replace(' ', "_")
        });

        Ok(Self {
            name,
            open: arguments.has_keyword(Keyword::Open),
            command: arguments.has_keyword(Keyword::Empty),
            items,
            attributes,
            tag,
            variable: flag,
        })
    }
}

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
            Self::Atom(..) => None,
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
        static WELL_KNOWN: OnceLock<CaseFoldMap<String, Element>> = OnceLock::new();

        validate(key, MxpError::InvalidElementName)?;

        if let Some(atom) = Atom::get(key) {
            Ok(ElementComponent::Atom(atom))
        } else if let Some(custom) = self.get(key) {
            Ok(ElementComponent::Custom(custom))
        } else if let Some(custom) = WELL_KNOWN.get_or_init(create_well_known_elements).get(key) {
            Ok(ElementComponent::Custom(custom))
        } else {
            Err(ParseError::new(key, MxpError::UnknownElement))
        }
    }
}

fn create_well_known_elements() -> CaseFoldMap<String, Element> {
    let color_atom = Atom::get("color").unwrap();
    let color_el = |color: &str| {
        let mut arguments = Arguments::new();
        arguments.push(color.to_ascii_lowercase());
        Element {
            name: format!("{color}MXP"),
            items: vec![ElementItem {
                atom: color_atom,
                arguments,
            }],
            attributes: Arguments::new(),
            tag: None,
            variable: None,
            open: true,
            command: false,
        }
    };
    [
        color_el("Black"),
        color_el("Red"),
        color_el("Green"),
        color_el("Yellow"),
        color_el("Blue"),
        color_el("Magenta"),
        color_el("Cyan"),
        color_el("White"),
    ]
    .into_iter()
    .map(|el| (CaseFold::new(el.name.clone()), el))
    .collect()
}
