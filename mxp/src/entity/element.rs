use std::num::NonZeroU8;
use std::str::FromStr;

use enumeration::{Enum, EnumSet};

use super::atom::{Atom, TagFlag};
use super::mode::Mode;
use crate::argument::scan::{Decoder, Scan};
use crate::argument::Arguments;
use crate::color::RgbColor;
use crate::keyword::ElementKeyword;
use crate::parser::{Error, ErrorKind, UnrecognizedVariant, Words};

/// List of arguments to an MXP tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementItem {
    pub atom: &'static Atom,
    pub arguments: Arguments,
}

impl ElementItem {
    pub fn parse(tag: &str) -> crate::Result<Self> {
        let mut words = Words::new(tag);
        let atom_name = words
            .next()
            .ok_or_else(|| Error::new(tag, ErrorKind::NoDefinitionTag))?;
        let invalid_name = match atom_name {
            "/" => Some(ErrorKind::DefinitionCannotCloseElement),
            "!" => Some(ErrorKind::DefinitionCannotDefineElement),
            _ => None,
        };
        if let Some(invalid) = invalid_name {
            return Err(Error::new(words.next().unwrap_or(""), invalid));
        }
        let atom = Atom::get(atom_name)
            .ok_or_else(|| Error::new(atom_name, ErrorKind::NoInbuiltDefinitionTag))?;
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
    pub fn from_str(text: &'a str) -> crate::Result<Self> {
        let tag = *text
            .as_bytes()
            .first()
            .ok_or_else(|| Error::new("collected element", ErrorKind::EmptyElement))?;

        match tag {
            b'!' => Ok(Self::Definition(&text[1..])),
            b'/' => Ok(Self::TagClose(&text[1..])),
            _ => Ok(Self::TagOpen(text)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum ParseAs {
    /// The text for the element is parsed by the automapper as the name of a room
    RoomName,
    /// he text for the element is parsed by the automapper as the description of a room
    RoomDesc,
    /// The text for the element is parsed by the automapper as exits for the room
    RoomExit,
    /// The text for the element is parsed by the automapper as a room number
    RoomNum,
    /// The text for the element is parsed by as a MUD Prompt
    Prompt,
}

impl FromStr for ParseAs {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "RoomName" => Self::RoomName,
            "RoomDesc" => Self::RoomDesc,
            "RoomExit" => Self::RoomExit,
            "RoomNum" => Self::RoomNum,
            "Prompt" => Self::Prompt,
            _ => return Err(Self::Err::new(s)),
        })
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
    /// Parsing flag
    pub parse_as: Option<ParseAs>,
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

    pub fn collect(text: &str) -> crate::Result<CollectedElement> {
        CollectedElement::from_str(text)
    }

    fn parse_items<S: AsRef<str>>(argument: Option<S>) -> crate::Result<Vec<ElementItem>> {
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
                return Err(Error::new(definitions, ErrorKind::NoTagInDefinition));
            }
            loop {
                let (end, endc) = iter
                    .next()
                    .ok_or_else(|| Error::new(definitions, ErrorKind::NoClosingDefinitionQuote))?;
                if endc == '>' {
                    let definition = &definitions[start + 1..end];
                    items.push(ElementItem::parse(definition)?);
                    break;
                }
                if (endc == '\'' || endc == '"') && !iter.any(|(_, c)| c == endc) {
                    return Err(Error::new(definitions, ErrorKind::NoClosingDefinitionQuote));
                }
            }
        }

        Ok(items)
    }

    pub fn parse<D: Decoder>(name: String, scanner: Scan<D>) -> crate::Result<Option<Self>> {
        let mut scanner = scanner.with_keywords();
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

        let (parse_as, variable) = match scanner.next_or(&["flag"])? {
            None => (None, None),
            Some(flag) => {
                let flag = flag.as_ref();
                if flag[.."set ".len()].eq_ignore_ascii_case("set ") {
                    (None, Some(flag["set ".len()..].to_owned()))
                } else {
                    (flag.parse().ok(), None)
                }
            }
        };

        let keywords = scanner.into_keywords();

        if keywords.contains(ElementKeyword::Delete) {
            return Ok(None);
        }

        Ok(Some(Self {
            name,
            open: keywords.contains(ElementKeyword::Open),
            command: keywords.contains(ElementKeyword::Empty),
            items,
            attributes,
            tag,
            parse_as,
            variable,
            fore: None,
            back: None,
            gag: false,
            window: None,
        }))
    }
}
