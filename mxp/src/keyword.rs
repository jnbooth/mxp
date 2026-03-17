use std::str::FromStr;
use std::{fmt, iter, slice};

use flagset::flags;

use crate::parse::UnrecognizedVariant;

flags! {
    /// Keywords for [`<DEST>`](crate::DEST) tags.
    pub(crate) enum DestKeyword: u8 {
        /// Causes the rest of the frame to be erased after displaying the text.
        Eof,
        /// Causes the rest of the line to be erased after displaying the text.
        Eol,
    }

    /// Keywords for [`<!ELEMENT>`](crate::Element) tags.
    pub(crate) enum ElementKeyword: u8 {
        Open,
        Empty,
        Delete,
    }

    /// Keywords for [`<!ENTITY>`](crate::Entity) tags.
    pub enum EntityKeyword: u8 {
        /// PRIVATE entities cannot be queried by the MUD client. They are completely hidden.
        Private,
        /// PUBLISH entities can be used by the client to produce a list of MUD Server variables to
        /// be accessed by the player.
        Publish,
        /// To delete an entity, use the DELETE argument. Setting an entity to a empty value does
        /// not delete it.
        Delete,
        /// The ADD argument causes the Value to be added as a new item in a string list. So, it is
        /// appended to the existing value of the variable. String lists are values separated by the
        /// `'|'` character.
        Add,
        /// The REMOVE argument causes the Value to be removed from the existing string list.
        Remove,
    }

    /// Keywords for [`<FRAME>`](crate::Frame) tags.
    pub(crate) enum FrameKeyword: u8 {
        Floating,
        Internal,
    }

    /// Keywords for [`<IMAGE>`](crate::Image) tags.
    pub(crate) enum ImageKeyword: u8 {
        IsMap,
    }

    /// Keywords for line tag updates.
    pub(crate) enum LineTagKeyword: u8 {
        Gag,
        Enable,
        Disable,
    }

    /// Keywords for [`<RELOCATE>`](crate::Relocate) tags.
    pub(crate) enum RelocateKeyword: u8 {
        Quiet,
    }

    /// Keywords for [`<SEND>`](crate::Link) tags.
    pub(crate) enum SendKeyword: u8 {
        Prompt,
    }
}

/// A trait for filtering out keywords from a list of strings.
///
/// `KeywordFilter` ensures that [`Decoder`]s do not parse element keywords as positional arguments.
/// It is used by [`ElementDecoder`] to resolve attribute entities (e.g. the `&col;` in
/// `<!ELEMENT boldtext '<COLOR &col;><B>' ATT='col=red'>`).
///
/// [`Decoder`]: crate::parse::Decoder
/// [`ElementDecoder`]: crate::element::ElementDecoder
pub trait KeywordFilter {
    /// The type of iterator produced by the filter.
    type Iter<'a, S: 'a>: Iterator<Item = &'a S>;

    /// Filters keywords out from a list of strings.
    fn iter<'a, S: AsRef<str> + 'a>(args: &'a [S]) -> Self::Iter<'a, S>;
}

/// Default `KeywordFilter` that iterates through strings without filtering them.
impl KeywordFilter for () {
    type Iter<'a, S: 'a> = slice::Iter<'a, S>;

    fn iter<'a, S: AsRef<str> + 'a>(args: &'a [S]) -> Self::Iter<'a, S> {
        args.iter()
    }
}

macro_rules! impl_keyword_filter {
    ($t:ty) => {
        impl KeywordFilter for $t {
            type Iter<'a, S: 'a> = iter::Filter<slice::Iter<'a, S>, fn(&&S) -> bool>;

            fn iter<'a, S: AsRef<str> + 'a>(args: &'a [S]) -> Self::Iter<'a, S> {
                args.iter()
                    .filter(|arg| <$t>::from_str(arg.as_ref()).is_err())
            }
        }
    };
}

impl_parse_enum!(DestKeyword, Eof, Eol);
impl_keyword_filter!(DestKeyword);

impl_parse_enum!(ElementKeyword, Open, Empty, Delete);
impl_keyword_filter!(ElementKeyword);

impl_parse_enum!(EntityKeyword, Private, Publish, Delete, Add, Remove);
impl_keyword_filter!(EntityKeyword);

impl_parse_enum!(FrameKeyword, Floating, Internal);
impl_keyword_filter!(FrameKeyword);

impl_parse_enum!(ImageKeyword, IsMap);
impl_keyword_filter!(ImageKeyword);

impl_parse_enum!(RelocateKeyword, Quiet);
impl_keyword_filter!(RelocateKeyword);

impl_parse_enum!(SendKeyword, Prompt);
impl_keyword_filter!(SendKeyword);

impl_parse_enum!(LineTagKeyword, Gag, Enable, Disable);
impl_keyword_filter!(LineTagKeyword);

impl fmt::Display for EntityKeyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Private => "PRIVATE",
            Self::Publish => "PUBLISH",
            Self::Delete => "DELETE",
            Self::Add => "ADD",
            Self::Remove => "REMOVE",
        }
        .fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_keywords() {
        let args = ["thing1", "open", "thing2", "empty", "empty", "thing3"]
            .into_iter()
            .collect::<Vec<&str>>();
        let non_keywords = <ElementKeyword as KeywordFilter>::iter(&args)
            .copied()
            .collect::<Vec<&str>>();
        assert_eq!(non_keywords, vec!["thing1", "thing2", "thing3"]);
    }
}
