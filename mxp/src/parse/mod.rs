mod error;
pub(crate) use error::StringVariant;
pub use error::{Error, UnrecognizedVariant};

mod error_kind;
pub use error_kind::ErrorKind;

mod from_str;
pub use from_str::FromStrError;
pub(crate) use from_str::{cleanup_source, parse_element};

mod scan;
pub(crate) use scan::{Decoder, Scan};

mod validation;
pub use validation::{is_valid, validate, validate_utf8};

mod words;
pub(crate) use words::Words;
