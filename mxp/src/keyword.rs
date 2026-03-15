use std::iter::FusedIterator;
use std::str::FromStr;
use std::{fmt, iter, slice};

use flagset::{FlagSet, Flags, flags};

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
        /// PUBLISH entities can be used by the client to produce a list of MUD Server variables to be access by the player.
        Publish,
        /// To delete an entity, use the DELETE argument. Setting an entity to a empty value does not delete it.
        Delete,
        /// The ADD argument causes the Value to be added as a new item in a string list. So, it is appended to the existing value of the variable. String lists are values separated by the | character.
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

    /// Keywords for [`<SEND>`](crate::Link) tags.
    pub(crate) enum SendKeyword: u8 {
        Prompt
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct KeywordFilterIter<K, I>
where
    K: Flags + FromStr,
    I: Iterator,
    I::Item: AsRef<str>,
{
    keywords: FlagSet<K>,
    inner: I,
}

impl<K, I> KeywordFilterIter<K, I>
where
    K: Flags + FromStr,
    I: Iterator,
    I::Item: AsRef<str>,
{
    pub fn new<T>(iter: T) -> Self
    where
        T: IntoIterator<IntoIter = I>,
    {
        Self {
            keywords: FlagSet::empty(),
            inner: iter.into_iter(),
        }
    }

    pub fn into_keywords(mut self) -> Result<FlagSet<K>, K::Err> {
        for item in self.inner {
            self.keywords |= K::from_str(item.as_ref())?;
        }
        Ok(self.keywords)
    }
}

impl<K, I> fmt::Debug for KeywordFilterIter<K, I>
where
    K: Flags + FromStr,
    I: Iterator + Clone,
    I::Item: AsRef<str> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<K, I> Iterator for KeywordFilterIter<K, I>
where
    K: Flags + FromStr,
    I: Iterator,
    I::Item: AsRef<str>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        for item in &mut self.inner {
            if let Ok(keyword) = K::from_str(item.as_ref()) {
                self.keywords |= keyword;
            } else {
                return Some(item);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.inner.size_hint();
        (0, upper)
    }
}

impl<K, I> DoubleEndedIterator for KeywordFilterIter<K, I>
where
    K: Flags + FromStr,
    I: DoubleEndedIterator,
    I::Item: AsRef<str>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.inner.next_back() {
            if let Ok(keyword) = K::from_str(next.as_ref()) {
                self.keywords |= keyword;
            } else {
                return Some(next);
            }
        }
        None
    }
}

impl<K, I> FusedIterator for KeywordFilterIter<K, I>
where
    K: Flags + FromStr,
    I: Iterator,
    I::Item: AsRef<str>,
{
}

/// A trait for filtering out keywords from a list of strings.
///
/// `KeywordFilter` is implemented for all types that implement [`FromStr`], filtering out strings
/// that can be parsed to the type.
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

impl_parse_enum!(SendKeyword, Prompt);
impl_keyword_filter!(SendKeyword);

impl_parse_enum!(LineTagKeyword, Gag, Enable, Disable);
impl_keyword_filter!(LineTagKeyword);

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
