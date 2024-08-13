use std::num::NonZeroU8;

use enumeration::EnumSet;

use super::atom::{Atom, TagFlag};
use super::mode::Mode;
use crate::argument::scan::{Decoder, Scan};
use crate::argument::{Arguments, Keyword};
use crate::color::RgbColor;
use crate::parser::{Error as MxpError, ParseError, Words};

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
            arguments: Arguments::parse(words)?,
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
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(text: &'a str) -> Result<Self, ParseError> {
        let tag = *text
            .as_bytes()
            .first()
            .ok_or_else(|| ParseError::new("collected element", MxpError::EmptyElement))?;

        match tag {
            b'!' => Ok(Self::Definition(&text[1..])),
            b'/' => Ok(Self::TagClose(&text[1..])),
            _ => Ok(Self::TagOpen(text)),
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
    pub tag: Option<NonZeroU8>,
    /// Which variable to set (SET x)
    pub variable: Option<String>,
    /// Whether the element is open (OPEN)
    pub open: bool,
    /// Whether the element has no closing tag (EMPTY)
    pub command: bool,
    /// Override foreground color (from line tag)
    pub fore: Option<RgbColor>,
    /// Override background color (from line tag)
    pub back: Option<RgbColor>,
    /// Suppress output in main window (from line tag)
    pub gag: bool,
    /// Redirect output to another window (from line tag)
    pub window: Option<String>,
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

    fn parse_items<S: AsRef<str>>(argument: Option<S>) -> Result<Vec<ElementItem>, ParseError> {
        let argument = match argument {
            Some(argument) => argument,
            None => return Ok(Vec::new()),
        };
        let definitions = argument.as_ref();
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

    pub fn parse<D: Decoder>(name: String, mut scanner: Scan<D>) -> Result<Self, ParseError> {
        let items = Self::parse_items(scanner.next()?)?;

        let attributes = match scanner.next_or(&["att"])? {
            Some(atts) => Arguments::parse(Words::new(atts.as_ref()))?,
            None => Arguments::default(),
        };

        const fn nonzero(n: Mode) -> NonZeroU8 {
            match NonZeroU8::new(n.0) {
                Some(n) => n,
                None => unreachable!(),
            }
        }
        const MIN_TAG: NonZeroU8 = nonzero(Mode::USER_DEFINED_MIN);
        const MAX_TAG: NonZeroU8 = nonzero(Mode::USER_DEFINED_MAX);

        let tag = match scanner
            .next_or(&["tag"])?
            .and_then(|s| s.as_ref().parse().ok())
        {
            Some(i) if !(MIN_TAG..=MAX_TAG).contains(&i) => None,
            tag => tag,
        };

        let flag = scanner.next_or(&["flag"])?.map(|flag| {
            let flag = flag.as_ref();
            flag.strip_prefix("set ")
                .unwrap_or(flag)
                .trim()
                .replace(' ', "_")
        });

        Ok(Self {
            name,
            open: scanner.has_keyword(Keyword::Open),
            command: scanner.has_keyword(Keyword::Empty),
            items,
            attributes,
            tag,
            variable: flag,
            fore: None,
            back: None,
            gag: false,
            window: None,
        })
    }
}
