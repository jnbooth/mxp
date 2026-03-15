use std::fmt;
use std::ops::Deref;

use crate::arguments::Arguments;

/// A simple wrapper around `&str` used in [`AttributeListDefinition`] and [`ParsedTagOpen`] to hint
/// that the underlying string slice should be parsed as [`Arguments`].
///
/// [`AttributeListDefinition`]: crate::parsed::AttributeListDefinition
/// [`ParsedTagOpen`]: crate::parsed::ParsedTagOpen
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArgumentsStr<'a>(pub &'a str);

impl<'a> ArgumentsStr<'a> {
    /// Attempts to parse the source data as `Arguments`.
    #[inline]
    pub fn parse_args(self) -> crate::Result<Arguments<'a>> {
        Arguments::parse(self.0)
    }

    /// Unwraps the newtype, returning the underlying string slice.
    #[inline]
    pub const fn into_inner(self) -> &'a str {
        self.0
    }
}

impl fmt::Debug for ArgumentsStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for ArgumentsStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for ArgumentsStr<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl AsRef<str> for ArgumentsStr<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> From<ArgumentsStr<'a>> for &'a str {
    #[inline]
    fn from(value: ArgumentsStr<'a>) -> Self {
        value.0
    }
}

impl<'a> From<&'a str> for ArgumentsStr<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

impl PartialEq<str> for ArgumentsStr<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        *self.0 == *other
    }
}
impl PartialEq<ArgumentsStr<'_>> for str {
    #[inline]
    fn eq(&self, other: &ArgumentsStr) -> bool {
        *self == *other.0
    }
}
impl PartialEq<String> for ArgumentsStr<'_> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        *self.0 == *other.as_str()
    }
}
impl PartialEq<ArgumentsStr<'_>> for String {
    #[inline]
    fn eq(&self, other: &ArgumentsStr<'_>) -> bool {
        *self.as_str() == *other.0
    }
}
