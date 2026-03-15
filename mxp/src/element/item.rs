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
    pub fn parse(tag: &str) -> crate::Result<Self> {
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
            return Err(Error::new(tag, invalid));
        }
        let tag = Tag::well_known(tag_name)
            .ok_or_else(|| Error::new(tag_name, ErrorKind::NoInbuiltDefinitionTag))?;
        Ok(Self {
            tag,
            arguments: words.parse_args_to_owned()?,
        })
    }

    pub fn parse_all(argument: &str) -> crate::Result<Vec<Self>> {
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
}
