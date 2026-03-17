use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TryFromNodeError {
    pub(crate) prefix: &'static str,
    pub(crate) expected: &'static str,
    pub(crate) got: &'static str,
}

impl TryFromNodeError {
    pub fn expected_type(&self) -> String {
        format!("{}::{}", self.prefix, self.expected)
    }

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
