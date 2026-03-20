mod error;
pub(crate) use error::StringVariant;
pub use error::{Error, UnrecognizedVariant};

mod error_kind;
pub use error_kind::ErrorKind;

mod from_str;
pub use from_str::FromStrError;
pub(crate) use from_str::{cleanup_source, parse_element};

mod into_owned_string;
pub use into_owned_string::IntoOwnedString;

mod decoder;
pub use decoder::Decoder;
pub(crate) use decoder::{OwnedScan, Scan};

mod validation;
pub use validation::{is_valid, validate, validate_utf8};
pub(crate) use validation::{split_name, strip_terminating_slash};

mod argument_parser;
pub(crate) use argument_parser::ArgumentParser;

pub fn count_bytes(haystack: &[u8], needle: u8) -> usize {
    haystack.iter().fold(0, |n, c| n + u32::from(*c == needle)) as usize
}
