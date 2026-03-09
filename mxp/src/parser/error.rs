use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;
use std::str;

use flagset::Flags;

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
    /// variable name in FLAG does not meet MUSHclient rules
    BadVariableName,
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
    /// eg. <!ELEMENT foo '<send &bar>'>
    NoClosingSemicolonInArgument,
    /// closing tag we don't recognise
    ClosingUnknownTag,
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
    /// invalid argument to <support> tag
    InvalidSupportArgument,
    /// invalid argument to <option> tag
    InvalidOptionArgument,
    /// eg. <!ELEMENT foo '</bold>' >
    DefinitionCannotCloseElement,
    /// eg. <!ELEMENT foo '<!ELEMENT>' >
    DefinitionCannotDefineElement,
    /// cannot change option with <recommend_option>
    CannotChangeOption,
    /// option not in acceptable range
    OptionOutOfRange,
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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: \"{}\"", self.error, self.target)
    }
}

impl std::error::Error for Error {}

impl Error {
    /// Constructs an error of the specified kind from a stringlike target being parsed.
    pub fn new<T: ParseErrorTarget>(target: T, error: ErrorKind) -> Self {
        Self {
            target: target.into_target(),
            error,
        }
    }
}

/// Types that can be used to construct an [`mxp::Error`](Error).
pub trait ParseErrorTarget {
    fn into_target(self) -> String;
}

impl ParseErrorTarget for String {
    fn into_target(self) -> String {
        self
    }
}

impl ParseErrorTarget for Cow<'_, str> {
    fn into_target(self) -> String {
        self.into_owned()
    }
}

impl ParseErrorTarget for &String {
    fn into_target(self) -> String {
        self.clone()
    }
}

impl ParseErrorTarget for &str {
    fn into_target(self) -> String {
        self.to_owned()
    }
}

impl ParseErrorTarget for &[u8] {
    fn into_target(self) -> String {
        String::from_utf8_lossy(self).into_owned()
    }
}

impl ParseErrorTarget for &Vec<u8> {
    fn into_target(self) -> String {
        String::from_utf8_lossy(self).into_owned()
    }
}

pub(crate) trait StringVariant: Sized + 'static {
    type Variant: fmt::Debug;
    const VARIANTS: &[Self::Variant];
}

impl<T: Flags + fmt::Debug> StringVariant for T {
    type Variant = Self;
    const VARIANTS: &[Self] = Self::LIST;
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

impl<T> PartialEq<UnrecognizedVariant<T>> for UnrecognizedVariant<T> {
    fn eq(&self, other: &UnrecognizedVariant<T>) -> bool {
        self.input == other.input
    }
}

impl<T> Eq for UnrecognizedVariant<T> {}

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
