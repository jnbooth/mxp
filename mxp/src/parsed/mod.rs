//! Typed representations of parsed MXP strings.

mod definition;
pub use definition::{
    AttributeListDefinition, ParsedDefinition, ParsedElementDefinition, ParsedEntityDefinition,
    ParsedLineTagDefinition,
};

mod element;
pub use element::{ParsedElement, ParsedTagClose, ParsedTagOpen};
