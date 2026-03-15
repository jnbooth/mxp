use std::fmt;

/// Type associated with an [`mxp::Error`](crate::Error).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// `&#5000;`
    IllegalEntityNumber,
    /// `&*;`
    InvalidEntityName,
    /// `&#xxx;`
    InvalidEntityNumber,
    /// `<!ENTITY foo &quot>`
    NoClosingSemicolon,
    /// `&foo;`
    UnknownEntity,

    /// `<>`
    EmptyElement,
    /// `<!>`, `<!ATTLIST>`
    IncompleteElement,
    /// `<send>` in open mode
    UnsecuredElement,
    /// `<2345>`, `</2345>`
    InvalidElementName,
    /// `<foo>`
    UnknownElement,

    /// `</send bar>`
    ArgumentsToClosingTag,
    /// when closing an open tag secure tag blocks it
    OpenTagBlockedBySecureTag,
    /// cannot close tag - it was opened in secure mode
    TagOpenedInSecureMode,
    /// `<i></i></bold>`
    UnmatchedCloseTag,

    /// cannot redefine inbuilt entity
    CannotRedefineEntity,
    /// `<!FOO ...>`
    InvalidDefinition,
    /// `<!TAG 3>`
    IllegalLineTag,
    /// `<!ATTLIST foo>`
    UnknownElementInAttlist,
    /// `<!ELEMENT ... >` in open mode
    UnsecuredDefinition,

    /// `<!ELEMENT foo '</bold>'>`
    CloseTagInDefinition,
    /// `<!ELEMENT foo '<!ELEMENT>'>`
    DefinitionInDefinition,
    /// `<!ELEMENT foo '<>'>`
    EmptyElementInDefinition,
    /// `<!ELEMENT TAG=0>`
    IllegalLineTagInDefinition,
    /// `<!ELEMENT foo 'bold'>` (should be `<bold>`)
    NoTagInDefinition,
    /// `<!ELEMENT foo '<bar>'>`
    UnknownElementInDefinition,
    /// `<!ELEMENT foo '<<bold>'>`
    UnexpectedSymbolInDefinition,
    /// `<!ELEMENT foo '<bold'>`
    UnterminatedElementInDefinition,
    /// `<!ELEMENT foo '<send "west>'>`
    UnterminatedQuoteInDefinition,

    /// `<font color=>`
    EmptyArgument,
    /// `<a>`
    MissingArguments,
    /// `<color 123=...>`
    InvalidArgumentName,
    /// `12d4`
    InvalidNumber,
    /// `<send foo>`
    UnexpectedArgument,
    /// `octarine`
    UnknownColor,

    /// cannot convert bytes into UTF-8
    InvalidUtf8,

    /// `<!-- ... \n`
    UnterminatedComment,
    /// `< ... \n`
    UnterminatedElement,
    /// `& ... \n`
    UnterminatedEntity,
    /// `< ' ... \n`
    UnterminatedQuote,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IllegalEntityNumber => "entity number out of bounds",
            Self::InvalidEntityName => "invalid name for entity",
            Self::InvalidEntityNumber => "invalid entity number",
            Self::NoClosingSemicolon => "entity missing semicolon",
            Self::UnknownEntity => "unrecognized entity",
            Self::EmptyElement => "received empty element",
            Self::IncompleteElement => "incomplete element",
            Self::UnsecuredElement => "received secure element in open mode",
            Self::InvalidElementName => "invalid name for element",
            Self::UnknownElement => "unrecognized element",
            Self::ArgumentsToClosingTag => "arguments inside closing tag",
            Self::OpenTagBlockedBySecureTag => "open tag blocked from closing by secure tag",
            Self::TagOpenedInSecureMode => {
                "received closing tag in open mode for element opened in secure mode"
            }
            Self::UnmatchedCloseTag => "received close tag without matching open tag",
            Self::CannotRedefineEntity => "cannot redefine global entity",
            Self::InvalidDefinition => "invalid definition type",
            Self::IllegalLineTag => "mode out of bounds for user-defined line tags",
            Self::UnknownElementInAttlist => "unrecognized element in attribute list",
            Self::UnsecuredDefinition => "received definition in open mode",
            Self::CloseTagInDefinition => "closing tag inside element definition",
            Self::DefinitionInDefinition => "definition inside element definition",
            Self::EmptyElementInDefinition => "empty element in element definition",
            Self::IllegalLineTagInDefinition => {
                "mode in element definition out of bounds for user-defined line tags"
            }
            Self::NoTagInDefinition => "element definition without tags",
            Self::UnknownElementInDefinition => "unrecognized element in element definition",
            Self::UnexpectedSymbolInDefinition => "unexpected '<' inside element definition",
            Self::UnterminatedElementInDefinition => "unterminated element in element definition",
            Self::UnterminatedQuoteInDefinition => "unterminated quote in element definition",
            Self::EmptyArgument => "empty argument",
            Self::MissingArguments => "missing arguments for element",
            Self::InvalidArgumentName => "invalid name for argument",
            Self::InvalidNumber => "invalid number",
            Self::UnexpectedArgument => "found unexpected argument",
            Self::UnknownColor => "unrecognized color",
            Self::InvalidUtf8 => "invalid UTF-8",
            Self::UnterminatedComment => "reached end of line without terminating comment",
            Self::UnterminatedElement => "reached end of line without terminating element",
            Self::UnterminatedEntity => "reached end of line without terminating entity",
            Self::UnterminatedQuote => "reached end of line without terminating quote",
        }
        .fmt(f)
    }
}
