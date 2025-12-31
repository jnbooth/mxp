use std::borrow::Cow;
use std::num::NonZero;
use std::str::FromStr;

use super::mode::Mode;
use super::tag::{Tag, Tags};
use crate::argument::{Arguments, Decoder, Scan};
use crate::color::RgbColor;
use crate::keyword::ElementKeyword;
use crate::parser::{Error, ErrorKind, UnrecognizedVariant, Words};

/// List of arguments to an MXP tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementItem<S: AsRef<str>> {
    pub tag: &'static Tag,
    pub arguments: Arguments<S>,
}

impl<S: AsRef<str>> ElementItem<S> {
    pub(crate) fn parse<'a>(tag: &'a str, tags: &Tags) -> crate::Result<Self>
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
        let tag = tags
            .get(tag_name)
            .ok_or_else(|| Error::new(tag_name, ErrorKind::NoInbuiltDefinitionTag))?;
        Ok(Self {
            tag,
            arguments: words.parse_args()?,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DefinitionKind {
    AttributeList,
    Element,
    Entity,
    LineTag,
}

impl FromStr for DefinitionKind {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match_ci! {s,
            "attlist" | "att" => Self::AttributeList,
            "element" | "el" => Self::Element,
            "entity" | "en" => Self::Entity,
            "tag" => Self::LineTag,
            _ => return Err(Self::Err::new(s))
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CollectedElement<'a> {
    Definition(CollectedDefinition<'a>),
    TagClose(&'a str),
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
            "roomname" => Self::RoomName,
            "roomdesc" => Self::RoomDesc,
            "roomexit" => Self::RoomExit,
            "roomnum" => Self::RoomNum,
            "prompt" => Self::Prompt,
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
    pub fn collect(text: &str) -> crate::Result<CollectedElement<'_>> {
        CollectedElement::from_str(text)
    }

    fn parse_items<S: AsRef<str>>(
        argument: Option<S>,
        tags: &Tags,
    ) -> crate::Result<Vec<ElementItem<String>>> {
        // Reduce monomorphization
        fn inner(argument: &str, tags: &Tags) -> crate::Result<Vec<ElementItem<String>>> {
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
                        items.push(ElementItem::parse(definition, tags)?);
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
        inner(argument.as_ref(), tags)
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

    pub(crate) fn parse<D: Decoder, S: AsRef<str>>(
        name: String,
        scanner: Scan<D, S>,
        tags: &Tags,
    ) -> crate::Result<Option<Self>> {
        let mut scanner = scanner.with_keywords();
        let items = Self::parse_items(scanner.next()?, tags)?;

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
