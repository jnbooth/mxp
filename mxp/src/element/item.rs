use std::borrow::Cow;
use std::fmt;

use super::action::Action;
use super::atomic_tag::AtomicTag;
use super::decoder::ElementDecoder;
use crate::arguments::Arguments;
use crate::parse::{Decoder, count_bytes, split_name, strip_terminating_slash, validate};
use crate::{Error, ErrorKind};

/// List of arguments to an MXP tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementItem {
    /// Standard atomic tag to apply. Determines the [`Action`](crate::Action) of this item.
    pub tag: &'static AtomicTag,
    /// Arguments for the tag. These may contain custom entities meant to be supplied as arguments
    /// to the custom element. An [`ElementDecoder`](crate::element::ElementDecoder) replaces
    /// those entities with the custom element's arguments.
    pub arguments: Arguments<'static, String>,
}

impl ElementItem {
    /// Resolves an `ElementItem` into an [`Action`] by decoding arguments and supplying them to the
    /// tag's definition.
    pub fn decode<'a, D>(
        &'a self,
        decoder: ElementDecoder<'a, D>,
    ) -> crate::Result<Action<Cow<'a, str>>>
    where
        D: Decoder,
    {
        Action::decode(self.tag.action, self.arguments.scan().with_decoder(decoder))
            .map_err(|e| e.with_context(format_args!(" for <{}>", self.tag.name)))
    }

    fn parse(source: &str) -> crate::Result<Self> {
        let (tag_name, body) = split_name(source);
        if tag_name.is_empty() {
            return Err(Error::new("", ErrorKind::EmptyElementInDefinition));
        }
        match tag_name.as_bytes() {
            [b'/', ..] => return Err(Error::braced(source, ErrorKind::CloseTagInDefinition)),
            [b'!', ..] => return Err(Error::braced(source, ErrorKind::DefinitionInDefinition)),
            _ => (),
        }
        let (args, _) = strip_terminating_slash(body);
        if let Some(tag) = AtomicTag::well_known(tag_name) {
            let arguments = args.parse()?;
            tag.check_arguments(&arguments)?;
            return Ok(Self { tag, arguments });
        }
        validate(tag_name, ErrorKind::InvalidElementName)?;
        Err(Error::new(tag_name, ErrorKind::UnknownElementInDefinition))
    }

    pub(crate) fn parse_all(source: &str) -> crate::Result<Vec<Self>> {
        let bytes = source.as_bytes();
        let size_guess = count_bytes(bytes, b'<');
        let mut items = Vec::with_capacity(size_guess);

        let mut iter = bytes.iter().enumerate();
        while let Some((start, &startc)) = iter.next() {
            if startc != b'<' {
                return Err(Error::new(source, ErrorKind::NoTagInDefinition));
            }
            loop {
                let (end, &endc) = iter.next().ok_or_else(|| {
                    Error::new(source, ErrorKind::UnterminatedElementInDefinition)
                })?;
                match endc {
                    b'<' => {
                        return Err(Error::new(source, ErrorKind::UnexpectedSymbolInDefinition));
                    }
                    b'>' => {
                        items.push(ElementItem::parse(&source[start + 1..end])?);
                        break;
                    }
                    b'\'' | b'"' if !iter.any(|(_, &c)| c == endc) => {
                        return Err(Error::new(source, ErrorKind::UnterminatedQuoteInDefinition));
                    }
                    _ => (),
                }
            }
        }

        Ok(items)
    }
}

impl fmt::Display for ElementItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Note: element definitions aren't decoded, so nothing here needs to be escaped.
        let Self { tag, arguments } = self;
        let name = tag.name;
        if arguments.is_empty() {
            write!(f, "<{name}>")
        } else {
            write!(f, "<{name} {arguments}>")
        }
    }
}
