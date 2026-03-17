//! Typed representations of parsed MXP strings.

mod definition;
pub use definition::{
    AttributeListDefinition, Definition, ElementDefinition, EntityDefinition, LineTagDefinition,
};

mod tag;
pub use tag::{Tag, TagBuilder, TagClose, TagOpen};
