mod error;
pub use error::{Error, ErrorKind, UnrecognizedVariant};

mod validation;
pub use validation::validate;

mod words;
pub use words::Words;
