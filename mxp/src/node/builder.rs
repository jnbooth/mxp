use std::borrow::{Borrow, Cow};

use html_escape::encode_double_quoted_attribute as escape;

use super::tag::TagOpen;
use crate::arguments::Arguments;

/// Utility for servers to build custom opening tags.
///
/// # Examples
///
/// ```
/// let mut builder = mxp::node::TagBuilder::new("monster");
/// builder
///     .push("hostile")
///     .push_all(&["Minotaur", "old veteran"])
///     .insert("hp", "150");
/// let tag = builder.build();
/// assert_eq!(
///     tag.to_string(),
///     r#"<monster hostile Minotaur "old veteran" hp=150>"#
/// );
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagBuilder<'a> {
    tag: TagOpen<'a, Cow<'a, str>>,
}

/// A server-side utility for building opening tags.
///
///
impl<'a> TagBuilder<'a> {
    /// Creates a builder for an opening tag with the specified element name.
    pub fn new(name: &'a str) -> Self {
        Self {
            tag: TagOpen {
                name,
                arguments: Arguments::default(),
            },
        }
    }

    /// Converts this builder into a finalized `TagOpen`.
    pub fn build(self) -> TagOpen<'a, Cow<'a, str>> {
        self.tag
    }

    /// Adds a positional argument or keyword without escaping special characters.
    pub fn push(&mut self, value: &'a str) -> &mut Self {
        self.tag.arguments.push(value.into());
        self
    }

    /// Adds a positional argument or keyword after escaping special characters.
    pub fn escape_and_push(&mut self, value: &'a str) -> &mut Self {
        self.tag.arguments.push(escape(value));
        self
    }

    /// Adds multiple positional arguments or keywords without escaping special characters.
    pub fn push_all<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Borrow<&'a str>,
    {
        self.tag
            .arguments
            .extend(iter.into_iter().map(|s| Cow::Borrowed(*s.borrow())));
        self
    }

    /// Adds multiple positional arguments or keywords after escaping special characters.
    pub fn escape_and_push_all<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Borrow<&'a str>,
    {
        self.tag
            .arguments
            .extend(iter.into_iter().map(|s| escape(*s.borrow())));
        self
    }

    /// Inserts a named argument without escaping special characters.
    pub fn insert(&mut self, name: &'a str, value: &'a str) -> &mut Self {
        self.tag.arguments.insert(name, value.into());
        self
    }

    /// Inserts a named argument after escaping special characters.
    pub fn escape_and_insert(&mut self, name: &'a str, value: &'a str) -> &mut Self {
        self.tag.arguments.insert(name, escape(value));
        self
    }

    /// Inserts multiple named arguments without escaping special characters.
    pub fn insert_all<I, K, V>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: Borrow<&'a str>,
        V: Borrow<&'a str>,
    {
        self.tag.arguments.extend(iter.into_iter().map(|entry| {
            let (k, v) = entry.borrow();
            (*k.borrow(), Cow::Borrowed(*v.borrow()))
        }));
        self
    }

    /// Inserts multiple named arguments after escaping special characters.
    pub fn escape_and_insert_all<I, K, V>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: Borrow<&'a str>,
        V: Borrow<&'a str>,
    {
        self.tag.arguments.extend(iter.into_iter().map(|entry| {
            let (k, v) = entry.borrow();
            (*k.borrow(), escape(*v.borrow()))
        }));
        self
    }
}
