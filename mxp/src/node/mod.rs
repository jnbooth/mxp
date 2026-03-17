//! Typed representations of parsed MXP strings.

mod builder;
pub use builder::TagBuilder;

mod definition;
pub use definition::{
    AttributeListDefinition, Definition, ElementDefinition, EntityDefinition, LineTagDefinition,
};

mod error;
pub use error::TryFromNodeError;

mod tag;
pub use tag::{Tag, TagClose, TagOpen};
