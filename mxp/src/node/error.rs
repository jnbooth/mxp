use std::fmt;

/// The error type returned when a conversion from a sum type to a specific node type fails.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TryFromNodeError {
    pub(crate) prefix: &'static str,
    pub(crate) expected: &'static str,
    pub(crate) got: &'static str,
}

impl TryFromNodeError {
    /// The type that was requested for conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::node::{Tag, TagClose, TagOpen};
    ///
    /// let node = Tag::Close(TagClose { name: "a" });
    /// let err = TagOpen::try_from(node).unwrap_err();
    /// assert_eq!(err.expected_type(), "Tag::Open");
    /// ```
    pub fn expected_type(&self) -> String {
        format!("{}::{}", self.prefix, self.expected)
    }

    /// The actual type that was found.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::node::{Tag, TagClose, TagOpen};
    ///
    /// let node = Tag::Close(TagClose { name: "a" });
    /// let err = TagOpen::try_from(node).unwrap_err();
    /// assert_eq!(err.received_type(), "Tag::Close");
    /// ```
    pub fn received_type(&self) -> String {
        format!("{}::{}", self.prefix, self.got)
    }
}

impl fmt::Display for TryFromNodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            prefix,
            expected,
            got,
        } = self;
        write!(f, "expected {prefix}::{expected}, got {prefix}::{got}")
    }
}
