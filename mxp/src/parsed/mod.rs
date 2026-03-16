//! Typed representations of parsed MXP strings.

mod arguments_str;
pub use arguments_str::ArgumentsStr;

mod definition;
pub use definition::{
    AttributeListDefinition, ParsedDefinition, ParsedElementDefinition, ParsedEntityDefinition,
    ParsedLineTagDefinition,
};

mod element;
pub use element::{ParsedElement, ParsedTagClose, ParsedTagOpen};
