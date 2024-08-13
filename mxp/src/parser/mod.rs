mod error;
pub use error::{Error, ErrorKind};

mod validation;
pub use validation::{is_valid, validate};

mod words;
pub use words::Words;
