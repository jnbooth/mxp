mod error;
pub(crate) use error::StringVariant;
pub use error::{Error, ErrorKind, ParseErrorTarget, UnrecognizedVariant};

mod validation;
pub use validation::{is_valid, validate};

mod words;
pub use words::Words;
