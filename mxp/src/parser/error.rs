use std::fmt::{self, Debug, Display, Formatter};
use std::marker::PhantomData;
use std::str;

use enumeration::Enum;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum ErrorKind {
    ///  eg. < ... \n
    UnterminatedElement,
    ///  eg. <!-- ... \n
    UnterminatedComment,
    ///  eg. & ... \n
    UnterminatedEntity,
    ///  eg. < ' ... \n
    UnterminatedQuote,
    ///  eg. <>
    EmptyElement,
    ///  eg. <!>
    ElementTooShort,
    ///  eg. &*;
    InvalidEntityName,
    ///  eg. <!ELEMENT ... > in open mode
    DefinitionWhenNotSecure,
    ///  eg. < 2345 >  or </ 2345 >
    InvalidElementName,
    ///  ie. not <!ELEMENT ...> or <!ENTITY ...>
    InvalidDefinition,
    ///  cannot redefine inbuilt element
    CannotRedefineElement,
    ///  no < in element definition, eg. <!ELEMENT foo 'bold' >  (should be '<bold>')
    NoTagInDefinition,
    ///  eg. <!ELEMENT foo '<<bold>' >
    UnexpectedDefinitionSymbol,
    ///  eg. <!ELEMENT foo '<send "west>' >
    NoClosingDefinitionQuote,
    ///  eg. <!ELEMENT foo '<bold' >
    NoClosingDefinitionTag,
    ///  defining unknown tag, eg. <!ELEMENT foo '<bar>' >
    NoInbuiltDefinitionTag,
    ///  eg. <!ELEMENT foo '<>' >
    NoDefinitionTag,
    ///  variable name in FLAG does not meet MUSHclient rules
    BadVariableName,
    ///  ATTLIST for undefined element name
    UnknownElementInAttlist,
    ///  cannot redefine inbuilt entity
    CannotRedefineEntity,
    ///  eg. <!ENTITY foo &quot >
    NoClosingSemicolon,
    ///  eg. <!ENTITY foo 'bar' xxxx >
    UnexpectedEntityArguments,
    ///  eg. <blah>
    UnknownElement,
    ///  eg. <send> in open mode
    ElementWhenNotSecure,
    ///  eg. <!ELEMENT foo '<send &bar>'>
    NoClosingSemicolonInArgument,
    ///  closing tag we don't recognise
    ClosingUnknownTag,
    ///  argument to COLOR or FONT not recognised color
    UnknownColor,
    /// eg. 12d4
    InvalidNumber,
    ///  eg. &#xxx;
    InvalidEntityNumber,
    ///  eg. &#5000;
    DisallowedEntityNumber,
    ///  eg. &foo;
    UnknownEntity,
    ///  eg. <color 123=blue>  (123 is invalid)
    InvalidArgumentName,
    ///  eg. <font color=>
    NoArgument,
    /// eg. <a>
    IncompleteArguments,
    ///  invalid argument to <support> tag
    InvalidSupportArgument,
    ///  invalid argument to <option> tag
    InvalidOptionArgument,
    ///  eg. <!ELEMENT foo '</bold>' >
    DefinitionCannotCloseElement,
    ///  eg. <!ELEMENT foo '<!ELEMENT>' >
    DefinitionCannotDefineElement,
    ///  cannot change option with <recommend_option>
    CannotChangeOption,
    ///  option not in acceptable range
    OptionOutOfRange,
    /// cannot convert bytes into UTF-8
    MalformedBytes,
    ///  eg. </send bar >
    ArgumentsToClosingTag,
    ///  when closing an open tag secure tag blocks it
    OpenTagBlockedBySecureTag,
    ///  eg. </bold> when no opening tag
    OpenTagNotThere,
    ///  cannot close tag - it was opened in secure mode
    TagOpenedInSecureMode,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error {
    target: String,
    error: ErrorKind,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}: \"{}\"", self.error, self.target)
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn new<T: ParseErrorTarget>(target: T, error: ErrorKind) -> Self {
        Self {
            target: target.into_target(),
            error,
        }
    }
}

pub trait ParseErrorTarget {
    fn into_target(self) -> String;
}

impl ParseErrorTarget for String {
    fn into_target(self) -> String {
        self
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

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnrecognizedVariant<T> {
    input: String,
    __marker: PhantomData<T>,
}

impl<T> UnrecognizedVariant<T> {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_owned(),
            __marker: PhantomData,
        }
    }
}

impl<T: Debug + Enum> Display for UnrecognizedVariant<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut range = T::enumerate(..);
        write!(f, "got {}, expected one of: {:?}", self.input, range.next())?;
        for variant in range {
            write!(f, ", {variant:?}")?;
        }
        Ok(())
    }
}

impl<T: Debug + Enum> std::error::Error for UnrecognizedVariant<T> {}
