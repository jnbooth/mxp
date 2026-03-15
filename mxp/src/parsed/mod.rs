//! Typed representations of parsed MXP strings.

mod arguments_str;
pub use arguments_str::ArgumentsStr;

mod definition;
pub use definition::{
    AttributeListDefinition, ElementDefinition, EntityDefinition, LineTagDefinition,
    ParsedDefinition,
};

mod element;
pub use element::{ParsedElement, ParsedTagClose, ParsedTagOpen};
