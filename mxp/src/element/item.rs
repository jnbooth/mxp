use super::tag::Tag;
use crate::parse::{Arguments, Words};
use crate::{Error, ErrorKind};

/// List of arguments to an MXP tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementItem<'a> {
    pub tag: &'static Tag,
    pub arguments: Arguments<'a>,
}

impl ElementItem<'static> {
    pub(crate) fn parse(tag: &str) -> crate::Result<Self> {
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
}
