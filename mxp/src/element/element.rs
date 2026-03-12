use std::borrow::Cow;
use std::num::NonZero;

use super::collected::CollectedElement;
use super::decoder::DecodeElement;
use super::item::ElementItem;
use super::parse_as::ParseAs;
use crate::color::RgbColor;
use crate::keyword::ElementKeyword;
use crate::mode::Mode;
use crate::parse::{Arguments, Decoder, Error, ErrorKind, Words};

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
    pub items: Vec<ElementItem>,
    /// List of attributes to this element (ATT="xx")
    pub attributes: Arguments<'static>,
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
    pub fn collect(source: &str, secure: bool) -> crate::Result<CollectedElement<'_>> {
        CollectedElement::parse(source, secure)
    }

    pub fn decode<'a, D>(&'a self, args: &'a Arguments<'a>, decoder: D) -> DecodeElement<'a, D>
    where
        D: Decoder + Copy,
    {
        DecodeElement::new(self, args, decoder)
    }

    /// Parses an MXP element from a definition, using the specified entity map for decoding.
    pub fn parse<D: Decoder>(definition: &str, decoder: D) -> crate::Result<ElementCommand> {
        let mut words = Words::new(definition);
        let name = words.validate_next_or(ErrorKind::InvalidElementName)?;
        let args = words.parse_args_to_owned()?;

        let mut scanner = args.scan(decoder).with_keywords();
        let items = Self::parse_items(scanner.next()?.as_deref())?;

        let attributes = match scanner.next_or("att")? {
            Some(atts) => Words::new(&atts).parse_args_to_owned()?,
            None => Arguments::default(),
        };

        let tag = Self::parse_tag(scanner.next_or("tag")?)?;

        let (parse_as, variable) = match scanner.next_or("flag")? {
            None => (None, None),
            Some(flag) => {
                if flag[.."set ".len()].eq_ignore_ascii_case("set ") {
                    (None, Some(flag["set ".len()..].to_owned()))
                } else {
                    (flag.parse().ok(), None)
                }
            }
        };

        let keywords = scanner.into_keywords()?;

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

    pub(crate) fn well_known() -> [(String, Element); 8] {
        fn color_el(name: &'static str, hex: u32) -> (String, Element) {
            (
                name.to_owned(),
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
    }

    fn parse_items(argument: Option<&str>) -> crate::Result<Vec<ElementItem>> {
        let Some(argument) = argument else {
            return Ok(Vec::new());
        };
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
                    .ok_or_else(|| Error::new(argument, ErrorKind::NoClosingDefinitionTag))?;
                match endc {
                    '<' => return Err(Error::new(argument, ErrorKind::UnexpectedDefinitionSymbol)),
                    '>' => {
                        let definition = &argument[start + 1..end];
                        items.push(ElementItem::parse(definition)?);
                        break;
                    }
                    '\'' | '"' if !iter.any(|(_, c)| c == endc) => {
                        return Err(Error::new(argument, ErrorKind::NoClosingDefinitionQuote));
                    }
                    _ => (),
                }
            }
        }

        Ok(items)
    }

    fn parse_tag(tag: Option<Cow<str>>) -> crate::Result<Option<NonZero<u8>>> {
        let Some(tag) = tag else {
            return Ok(None);
        };
        match tag.parse::<NonZero<u8>>() {
            Ok(tag) if Mode(tag.get()).is_user_defined() => Ok(Some(tag)),
            _ => Err(crate::Error::new(tag, ErrorKind::InvalidLineTag)),
        }
    }
}
