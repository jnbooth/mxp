use std::borrow::Cow;
use std::num::NonZero;
use std::str::FromStr;

use super::mode::Mode;
use super::tag::Tag;
use crate::argument::Arguments;
use crate::color::RgbColor;
use crate::entity::EntityMap;
use crate::keyword::ElementKeyword;
use crate::parser::{Error, ErrorKind, StringVariant, UnrecognizedVariant, Words};

/// List of arguments to an MXP tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementItem<S: AsRef<str>> {
    pub tag: &'static Tag,
    pub arguments: Arguments<S>,
}

impl<S: AsRef<str>> ElementItem<S> {
    pub(crate) fn parse<'a>(tag: &'a str) -> crate::Result<Self>
    where
        S: From<&'a str>,
    {
        let mut words = Words::new(tag);
        let tag_name = words
            .next()
            .ok_or_else(|| Error::new(tag, ErrorKind::NoDefinitionTag))?;
        let invalid_name = match tag_name {
            "/" => Some(ErrorKind::DefinitionCannotCloseElement),
            "!" => Some(ErrorKind::DefinitionCannotDefineElement),
            _ => None,
        };
        if let Some(invalid) = invalid_name {
            return Err(Error::new(words.next().unwrap_or(""), invalid));
        }
        let tag = Tag::well_known(tag_name)
            .ok_or_else(|| Error::new(tag_name, ErrorKind::NoInbuiltDefinitionTag))?;
        Ok(Self {
            tag,
            arguments: words.parse_args()?,
        })
    }
}

/// Type of MXP definition sent by the server.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DefinitionKind {
    /// [`<!ATTLIST>`](https://www.zuggsoft.com/zmud/mxp.htm#ATTLIST):
    /// Add attributes to elements.
    AttributeList,
    /// [`<!ELEMENT>`](https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT):
    /// Define a new [`Element`](crate::Element).
    Element,
    /// [`<!ENTITY>`](https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT):
    /// Define a new [`Entity`](crate::Entity).
    Entity,
    /// [`<!TAG>`](https://www.zuggsoft.com/zmud/mxp.htm#User-defined%20Line%20Tags):
    /// Change properties for a line tag.
    LineTag,
}

impl StringVariant for DefinitionKind {
    type Variant = &'static str;
    const VARIANTS: &[&str] = &["ATTLIST", "ATT", "ELEMENT", "EL", "ENTITY", "EN", "TAG"];
}

impl FromStr for DefinitionKind {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "attlist" | "att" => Ok(Self::AttributeList),
            "element" | "el" => Ok(Self::Element),
            "entity" | "en" => Ok(Self::Entity),
            "tag" => Ok(Self::LineTag),
            _ => Err(Self::Err::new(s))
        }
    }
}

/// MXP definition sent by the server, which may define an [attribute list], [element], [entity],
/// or [line tag].
///
/// [attribute list]: https://www.zuggsoft.com/zmud/mxp.htm#ATTLIST
/// [element]: https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT
/// [entity]: https://www.zuggsoft.com/zmud/mxp.htm#ENTITY
/// [line tag]: https://www.zuggsoft.com/zmud/mxp.htm#User-defined%20Line%20Tags
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CollectedDefinition<'a> {
    pub(crate) kind: DefinitionKind,
    pub(crate) text: &'a str,
}

impl<'a> CollectedDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    fn from_str(text: &'a str) -> crate::Result<Self> {
        fn fail_definition(text: &str) -> crate::Error {
            crate::Error::new(text, ErrorKind::InvalidDefinition)
        }

        let Some((kind, definition)) = text.split_once(' ') else {
            return Err(fail_definition(text));
        };
        let Ok(kind) = DefinitionKind::from_str(kind) else {
            return Err(fail_definition(text));
        };
        Ok(Self {
            kind,
            text: definition,
        })
    }
}

/// The three types of MXP tag elements sent by the server.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CollectedElement<'a> {
    /// A definition, e.g. [`<!ELEMENT>`].
    Definition(CollectedDefinition<'a>),
    /// A closing tag, e.g. [`</BOLD>`].
    TagClose(&'a str),
    /// An opening tag, e.g. [`<BOLD>`].
    TagOpen(&'a str),
}

impl<'a> CollectedElement<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn from_str(text: &'a str) -> crate::Result<Self> {
        let tag = *text
            .as_bytes()
            .first()
            .ok_or_else(|| Error::new("collected element", ErrorKind::EmptyElement))?;

        match tag {
            b'!' => Ok(Self::Definition(CollectedDefinition::from_str(&text[1..])?)),
            b'/' => Ok(Self::TagClose(&text[1..])),
            _ => Ok(Self::TagOpen(text)),
        }
    }
}

/// The MUD server can tag a line to be parsed in a specific way by the client.
///
/// See [MXP specification: MXP Line Tags](https://www.zuggsoft.com/zmud/mxp.htm#MXP%20Line%20Tags).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ParseAs {
    /// The text for the element is parsed by the automapper as the name of a room.
    RoomName,
    /// he text for the element is parsed by the automapper as the description of a room.
    RoomDesc,
    /// The text for the element is parsed by the automapper as exits for the room.
    RoomExit,
    /// The text for the element is parsed by the automapper as a room number.
    RoomNum,
    /// The text for the element is parsed by as a MUD Prompt.
    Prompt,
}

impl StringVariant for ParseAs {
    type Variant = Self;
    const VARIANTS: &[Self] = &[
        Self::RoomName,
        Self::RoomDesc,
        Self::RoomExit,
        Self::RoomNum,
        Self::Prompt,
    ];
}

impl FromStr for ParseAs {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "roomname" => Ok(Self::RoomName),
            "roomdesc" => Ok(Self::RoomDesc),
            "roomexit" => Ok(Self::RoomExit),
            "roomnum" => Ok(Self::RoomNum),
            "prompt" => Ok(Self::Prompt),
            _ => Err(Self::Err::new(s)),
        }
    }
}

/// Result of [`Element::parse`].
#[derive(Debug)]
pub enum ElementCommand {
    /// The server is commanding the client to define an element.
    Define(Element),
    /// The server is commanding the client to delete an element with the specified name.
    Delete(String),
}

/// User-defined MXP tags that we recognise, e.g. <boldcolor>.
/// For example: <!ELEMENT boldtext '<COLOR &col;><B>' ATT='col=red'>
///
/// See [MXP specification: Elements](https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Element {
    /// Tag name
    pub name: String,
    /// What atomic elements it defines (arg 1)
    pub items: Vec<ElementItem<String>>,
    /// List of attributes to this element (ATT="xx")
    pub attributes: Arguments<String>,
    /// Line tag number (20 - 99) (TAG=n)
    pub tag: Option<NonZero<u8>>,
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
    /// Parses an element tag.
    pub fn collect(text: &str) -> crate::Result<CollectedElement<'_>> {
        CollectedElement::from_str(text)
    }

    /// Parses an MXP element from a definition, using the specified entity map for decoding.
    pub fn parse(definition: &str, entities: &EntityMap) -> crate::Result<ElementCommand> {
        let mut words = Words::new(definition);
        let name = words.validate_next_or(ErrorKind::InvalidElementName)?;
        let args = words.parse_args::<String>()?;

        let mut scanner = args.scan(entities).with_keywords();
        let items = Self::parse_items(scanner.next()?)?;

        let attributes = match scanner.next_or("att")? {
            Some(atts) => Words::new(atts.as_ref()).parse_args()?,
            None => Arguments::default(),
        };

        let tag = Self::parse_tag(scanner.next_or("tag")?)?;

        let (parse_as, variable) = match scanner.next_or("flag")? {
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
            return Ok(ElementCommand::Delete(name.to_owned()));
        }

        Ok(ElementCommand::Define(Self {
            name: name.to_owned(),
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

    fn parse_items<S: AsRef<str>>(argument: Option<S>) -> crate::Result<Vec<ElementItem<String>>> {
        // Reduce monomorphization
        fn inner(argument: &str) -> crate::Result<Vec<ElementItem<String>>> {
            let size_guess = argument.bytes().filter(|&c| c == b'<').count();
            let mut items = Vec::with_capacity(size_guess);

            let mut iter = argument.char_indices();
            while let Some((start, startc)) = iter.next() {
                if startc != '<' {
                    return Err(Error::new(argument, ErrorKind::NoTagInDefinition));
                }
                loop {
                    let (end, endc) = iter
                        .next()
                        .ok_or_else(|| Error::new(argument, ErrorKind::NoClosingDefinitionQuote))?;
                    if endc == '>' {
                        let definition = &argument[start + 1..end];
                        items.push(ElementItem::parse(definition)?);
                        break;
                    }
                    if (endc == '\'' || endc == '"') && !iter.any(|(_, c)| c == endc) {
                        return Err(Error::new(argument, ErrorKind::NoClosingDefinitionQuote));
                    }
                }
            }

            Ok(items)
        }
        let Some(argument) = argument else {
            return Ok(Vec::new());
        };

        inner(argument.as_ref())
    }

    fn parse_tag(tag: Option<Cow<str>>) -> crate::Result<Option<NonZero<u8>>> {
        let Some(tag) = tag else {
            return Ok(None);
        };
        match tag.as_ref().parse::<NonZero<u8>>() {
            Ok(tag) if Mode(tag.get()).is_user_defined() => Ok(Some(tag)),
            _ => Err(crate::Error::new(tag, ErrorKind::InvalidLineTag)),
        }
    }
}
