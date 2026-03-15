use super::tag::Tag;
use crate::parse::{Arguments, Words};
use crate::{Error, ErrorKind};

/// List of arguments to an MXP tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementItem {
    pub tag: &'static Tag,
    pub arguments: Arguments<'static, String>,
}

impl ElementItem {
    pub fn parse(source: &str) -> crate::Result<Self> {
        let mut words = Words::new(source);
        let tag_name = words
            .next()
            .ok_or_else(|| Error::new("", ErrorKind::EmptyElementInDefinition))?;
        match tag_name {
            "/" => return Err(Error::braced(source, ErrorKind::CloseTagInDefinition)),
            "!" => return Err(Error::braced(source, ErrorKind::DefinitionInDefinition)),
            _ => (),
        }
        if let Some(tag) = Tag::well_known(tag_name) {
            return Ok(Self {
                tag,
                arguments: words.try_into()?,
            });
        }
        crate::validate(tag_name, ErrorKind::InvalidElementName)?;
        Err(Error::new(tag_name, ErrorKind::UnknownElementInDefinition))
    }

    pub fn parse_all(source: &str) -> crate::Result<Vec<Self>> {
        let size_guess = source.bytes().filter(|&c| c == b'<').count();
        let mut items = Vec::with_capacity(size_guess);

        let mut iter = source.char_indices();
        while let Some((start, startc)) = iter.next() {
            if startc != '<' {
                return Err(Error::new(source, ErrorKind::NoTagInDefinition));
            }
            loop {
                let (end, endc) = iter.next().ok_or_else(|| {
                    Error::new(source, ErrorKind::UnterminatedElementInDefinition)
                })?;
                match endc {
                    '<' => {
                        return Err(Error::new(source, ErrorKind::UnexpectedSymbolInDefinition));
                    }
                    '>' => {
                        let definition = &source[start + 1..end];
                        items.push(ElementItem::parse(definition)?);
                        break;
                    }
                    '\'' | '"' if !iter.any(|(_, c)| c == endc) => {
                        return Err(Error::new(source, ErrorKind::UnterminatedQuoteInDefinition));
                    }
                    _ => (),
                }
            }
        }

        Ok(items)
    }
}
