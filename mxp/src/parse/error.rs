use std::fmt::{self, Write};
use std::marker::PhantomData;
use std::string::FromUtf8Error;

use crate::ErrorKind;

/// Error caused by attempting to parse malformed MXP data from the server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    target: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Error { kind, target } = self;
        if target.is_empty() {
            write!(f, "{kind}")
        } else {
            write!(f, "{kind}: \"{target}\"")
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    /// Constructs an error of the specified kind from a stringlike target being parsed.
    pub fn new<T: Into<String>>(target: T, error: ErrorKind) -> Self {
        Self {
            kind: error,
            target: target.into(),
        }
    }

    pub(crate) fn braced(target: &str, error: ErrorKind) -> Self {
        Self {
            kind: error,
            target: format!("<{target}>"),
        }
    }

    /// Returns the kind of error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Appends additional context to the error message.
    #[must_use = "returns self"]
    pub fn with_context(mut self, context: fmt::Arguments<'_>) -> Self {
        self.target.write_fmt(context).expect("formatting error");
        self
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::new(
            String::from_utf8_lossy(value.as_bytes()).into_owned(),
            ErrorKind::InvalidUtf8,
        )
    }
}

pub(crate) trait StringVariant: Sized + 'static {
    const VARIANTS: &[&str];
}

/// Error caused by attempting to parse a string that did not match any variant of a string-like
/// type.
pub struct UnrecognizedVariant<T> {
    input: String,
    phantom: PhantomData<T>,
}

impl<T> UnrecognizedVariant<T> {
    pub(crate) fn new(input: &str) -> Self
    where
        T: StringVariant,
    {
        Self {
            input: input.to_owned(),
            phantom: PhantomData,
        }
    }

    pub fn target(&self) -> &str {
        &self.input
    }
}

impl<T> Clone for UnrecognizedVariant<T> {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            phantom: self.phantom,
        }
    }
}

impl<T> fmt::Debug for UnrecognizedVariant<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("UnrecognizedVariant")
            .field(&self.input)
            .finish()
    }
}

impl<T: StringVariant> fmt::Display for UnrecognizedVariant<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut range = T::VARIANTS.iter();
        let first_variant = range.next().unwrap();
        write!(f, "got {}, expected one of: {first_variant:?}", self.input)?;
        for variant in range {
            write!(f, ", {variant:?}")?;
        }
        Ok(())
    }
}

impl<T: StringVariant> std::error::Error for UnrecognizedVariant<T> {}

impl<T: StringVariant> From<UnrecognizedVariant<T>> for Error {
    fn from(value: UnrecognizedVariant<T>) -> Self {
        Self::new(value.input, ErrorKind::UnexpectedArgument)
    }
}
