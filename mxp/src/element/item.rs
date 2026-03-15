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
    fn parse(source: &str) -> crate::Result<Self> {
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
            let arguments = words.try_into()?;
            tag.check_arguments(&arguments)?;
            return Ok(Self { tag, arguments });
        }
        crate::validate(tag_name, ErrorKind::InvalidElementName)?;
        Err(Error::new(tag_name, ErrorKind::UnknownElementInDefinition))
    }

    pub fn parse_all(source: &str) -> crate::Result<Vec<Self>> {
        let bytes = source.as_bytes();
        let size_guess = bytes.iter().filter(|c| **c == b'<').count();
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
