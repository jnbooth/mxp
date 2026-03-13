use std::fmt;
use std::marker::PhantomData;
use std::str;
use std::string::FromUtf8Error;

/// Type associated with an [`mxp::Error`](Error).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// eg. < ... \n
    UnterminatedElement,
    /// eg. <!-- ... \n
    UnterminatedComment,
    /// eg. & ... \n
    UnterminatedEntity,
    /// eg. < ' ... \n
    UnterminatedQuote,
    /// eg. <>
    EmptyElement,
    /// eg. <!>
    ElementTooShort,
    /// eg. &*;
    InvalidEntityName,
    /// eg. <!ELEMENT ... > in open mode
    DefinitionWhenNotSecure,
    /// eg. < 2345 >  or </ 2345 >
    InvalidElementName,
    /// ie. not <!ELEMENT ...> or <!ENTITY ...>
    InvalidDefinition,
    /// cannot redefine inbuilt element
    CannotRedefineElement,
    /// no < in element definition, eg. <!ELEMENT foo 'bold' >  (should be '<bold>')
    NoTagInDefinition,
    /// eg. <!ELEMENT foo '<<bold>' >
    UnexpectedDefinitionSymbol,
    /// eg. <!ELEMENT foo '<send "west>' >
    NoClosingDefinitionQuote,
    /// eg. <!ELEMENT foo '<bold' >
    NoClosingDefinitionTag,
    /// defining unknown tag, eg. <!ELEMENT foo '<bar>' >
    NoInbuiltDefinitionTag,
    /// eg. <!ELEMENT foo '<>' >
    NoDefinitionTag,
    /// ATTLIST for undefined element name
    UnknownElementInAttlist,
    /// cannot redefine inbuilt entity
    CannotRedefineEntity,
    /// eg. <!ENTITY foo &quot >
    NoClosingSemicolon,
    /// eg. <!ENTITY foo 'bar' xxxx >
    UnexpectedEntityArguments,
    /// eg. <blah>
    UnknownElement,
    /// eg. <send> in open mode
    ElementWhenNotSecure,
    /// argument to COLOR or FONT not recognised color
    UnknownColor,
    /// eg. 12d4
    InvalidNumber,
    /// eg. &#xxx;
    InvalidEntityNumber,
    /// eg. &#5000;
    DisallowedEntityNumber,
    /// eg. &foo;
    UnknownEntity,
    /// eg. <color 123=blue>  (123 is invalid)
    InvalidArgumentName,
    /// eg. <font color=>
    NoArgument,
    /// eg. <a>
    IncompleteArguments,
    /// eg. <!ELEMENT foo '</bold>' >
    DefinitionCannotCloseElement,
    /// eg. <!ELEMENT foo '<!ELEMENT>' >
    DefinitionCannotDefineElement,
    /// cannot convert bytes into UTF-8
    MalformedBytes,
    /// eg. </send bar >
    ArgumentsToClosingTag,
    /// when closing an open tag secure tag blocks it
    OpenTagBlockedBySecureTag,
    /// eg. </bold> when no opening tag
    OpenTagNotThere,
    /// cannot close tag - it was opened in secure mode
    TagOpenedInSecureMode,
    /// eg. <!ELEMENT TAG=0>`
    InvalidLineTag,
}

/// Error caused by attempting to parse malformed MXP data from the server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    target: String,
    error: ErrorKind,
    source: Option<String>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: \"{}\"", self.error, self.target)
    }
}

impl std::error::Error for Error {}

impl Error {
    /// Constructs an error of the specified kind from a stringlike target being parsed.
    pub fn new<T: Into<String>>(target: T, error: ErrorKind) -> Self {
        Self {
            target: target.into(),
            error,
            source: None,
        }
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn set_source(&mut self, source: String) {
        self.source = Some(source);
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::new(
            String::from_utf8_lossy(value.as_bytes()).into_owned(),
            ErrorKind::MalformedBytes,
        )
    }
}

pub(crate) trait StringVariant: Sized + 'static {
    type Variant: fmt::Debug;
    const VARIANTS: &[Self::Variant];
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
        Self::new(value.input, ErrorKind::UnexpectedEntityArguments)
    }
}
