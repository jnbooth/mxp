use std::fmt;

/// Type associated with an [`mxp::Error`](crate::Error).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Valid entity names must start with an alphabetical letter and contain only alphanumeric
    /// characters, `_`, `-`, and `.`.
    ///
    /// Example: `&*;`
    InvalidEntityName,

    /// Entity string cannot be parsed to a positive decimal or hexadecimal integer.
    ///
    /// Example: `&#1x0;`
    InvalidEntityNumber,

    /// Entity is not terminated by a semicolon.
    ///
    /// Example: `<!ENTITY foo &quot>`
    NoClosingSemicolon,

    /// Entity is not recognized as either a global entity or a custom entity.
    ///
    /// Example: `&foo;`
    UnknownEntity,

    /// Received an element with no content.
    ///
    /// Example: `<>`
    EmptyElement,

    /// Received an element missing the rest of its content.
    ///
    /// Example: `<!>`, `<!ATTLIST>`
    IncompleteElement,

    /// Server sent a SECURE element, but the current line mode is OPEN.
    ///
    /// Example: `\x1B0z <var>`
    UnsecuredElement,

    /// Valid entity names must start with an alphabetical letter and contain only alphanumeric
    /// characters, `_`, `-`, and `.`.
    ///
    /// Example: `<2345>`
    InvalidElementName,

    /// Element is not recognized as either a standard (atomic) tag or a custom element.
    ///
    /// Example: `<foo>`
    UnknownElement,

    /// `</send bar>`
    ///
    /// Received a closing tag with arguments.
    ArgumentsToClosingTag,

    /// Due to receiving a newline character or line mode change from the server, the client
    /// attempted to close all unclosed OPEN tags. However, there was an unclosed secure tag in
    /// front of an unclosed OPEN tag.
    ///
    /// Example: `<b> <var> \n`
    OpenTagBlockedBySecureTag,

    /// While the line mode was OPEN, attempted to close a tag from a line that was not OPEN.
    ///
    /// Example: `\x1B[1z <var> \n \x1B[0z </var>`
    TagOpenedInSecureMode,

    /// `<i></i></bold>`
    UnmatchedCloseTag,

    /// Received something other than a '<' after setting
    /// [`Mode::SECURE_ONCE`](crate::Mode::SECURE_ONCE).
    ///
    /// Example: `\x1B[4z&quot;`
    TextAfterSecureOnce,

    /// Definition would overwrite a global XML entity.
    ///
    /// Example: `<!ENTITY lt lessthan>`
    CannotRedefineGlobalEntity,

    /// Definition tag (`<!...>`) does not begin begin with a recognized prefix
    /// (`!ATTLIST`, `!ATT`, `!ELEMENT`, `!EL`, `!ENTITY`, `!EN`, or `!TAG`).
    ///
    /// Example: `<!FOO ...>`
    InvalidDefinition,

    /// Tag definition declares a line that is not inside the legal range of user-defined line tags
    /// (20..=99).
    ///
    /// Example: `<!TAG 3>`
    IllegalLineTag,

    /// ATTLIST adds attributes to a custom element that was never defined.
    ///
    /// Example: `<!ATTLIST foo>`
    UnknownElementInAttlist,

    /// Received a definition while the line was in OPEN mode.
    UnsecuredDefinition,

    /// Element definition contains a closing tag.
    ///
    /// Example: `<!ELEMENT foo '</bold>'>`
    CloseTagInDefinition,

    /// Element definition contains a nested element definition.
    ///
    /// `<!ELEMENT foo '<!ELEMENT>'>`
    DefinitionInDefinition,

    /// Element definition contains an empty element.
    ///
    /// `<!ELEMENT foo '<>'>`
    EmptyElementInDefinition,

    /// Element definition defines a line tag that is not inside the legal range of user-defined
    /// line tags (20..=99).
    ///
    /// `<!ELEMENT TAG=3>`
    IllegalLineTagInDefinition,

    /// Element definition contains content not surrounded by tag brackets.
    ///
    /// Example: `<!ELEMENT foo 'bold'>`
    NoTagInDefinition,

    /// Element definition contains an element which is not recognized as either a standard
    /// (atomic) tag or a custom element.
    ///
    /// Example: `<!ELEMENT foo '<bar>'>`
    UnknownElementInDefinition,

    /// Element definition contains a superfluous opening tag bracket.
    ///
    /// Example: `<!ELEMENT foo '<<bold>'>`
    UnexpectedSymbolInDefinition,

    /// Element definition contains an element with an opening bracket but no closing bracket.
    ///
    /// Example: `<!ELEMENT foo '<bold'>`
    UnterminatedElementInDefinition,

    /// Element definition contains an unterminated quote.
    ///
    /// Example: `<!ELEMENT foo '<send "west>'>`
    UnterminatedQuoteInDefinition,

    /// Named attribute is not followed by a text value.
    ///
    /// Example: `<font color=>`
    ///
    /// Note: the proper way to use an empty string as an attribute value is via quotes, e.g.
    /// `<font color="">`.
    EmptyArgument,

    /// An element does not specify values for mandatory arguments.
    ///
    /// Example: `<a>`
    MissingArguments,

    /// Valid argument names must start with an alphabetical letter and contain only alphanumeric
    /// characters, `_`, `-`, and `.`.
    ///
    /// Example: `<color 123=...>`
    InvalidArgumentName,

    /// Numeric attribute cannot be parsed to an integer.
    ///
    /// Example: `12d4`
    InvalidNumber,

    /// Tag contents specify an attribute that does not belong to the element.
    ///
    /// Example: `<color octopus=green>`
    UnexpectedArgument,
    /// Attributes for a `<COLOR>`, `<FONT>`, or `<GAUGE>` specified an unrecognized color.
    /// Valid colors are hex codes (e.g. `#FF0000`) or names from the the standard list of
    /// [148 CSS colors], case-insensitive (e.g. `green`).
    ///
    /// Example: `<COLOR fore=octarine>`
    ///
    /// [148 CSS colors]: https://www.w3.org/wiki/CSS/Properties/color/keywords
    UnknownColor,

    /// Bytes are not valid UTF-8. This is a convenience error kind so that clients can handle UTF-8
    /// parsing while using `mxp::Result`.
    InvalidUtf8,

    /// Received a newline without terminating a comment. This is for clients to use.
    ///
    /// Example: `<!-- ... \n`
    UnterminatedComment,

    /// Received a newline without terminating an element. This is for clients to use.
    ///
    /// Example: `< ... \n`
    UnterminatedElement,

    /// Received a newline without terminating an entity. This is for clients to use.
    ///
    /// Example: `& ... \n`
    UnterminatedEntity,

    /// Received a newline without terminating a quote. This is for clients to use.
    ///
    /// Example: `< ' ... \n`
    UnterminatedQuote,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidEntityName => "invalid name for entity",
            Self::InvalidEntityNumber => "invalid entity number",
            Self::NoClosingSemicolon => "entity missing semicolon",
            Self::UnknownEntity => "unrecognized entity",
            Self::EmptyElement => "received empty element",
            Self::IncompleteElement => "incomplete element",
            Self::UnsecuredElement => "received secure element in OPEN mode",
            Self::InvalidElementName => "invalid name for element",
            Self::UnknownElement => "unrecognized element",
            Self::ArgumentsToClosingTag => "arguments inside closing tag",
            Self::OpenTagBlockedBySecureTag => "OPEN tag blocked from closing by SECURE tag",
            Self::TagOpenedInSecureMode => {
                "received closing tag in OPEN mode for element opened in SECURE mode"
            }
            Self::UnmatchedCloseTag => "received closing tag without matching opening tag",
            Self::TextAfterSecureOnce => {
                "received unexpected symbol after setting SECURE_ONCE mode"
            }
            Self::CannotRedefineGlobalEntity => "cannot redefine global entity",
            Self::InvalidDefinition => "invalid definition type",
            Self::IllegalLineTag => "mode out of bounds for user-defined line tags",
            Self::UnknownElementInAttlist => "unrecognized element in attribute list",
            Self::UnsecuredDefinition => "received definition in OPEN mode",
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
