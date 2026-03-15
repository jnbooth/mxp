mod definition;
pub use definition::{
    AttributeListDefinition, ElementDefinition, EntityDefinition, LineTagDefinition,
    ParsedDefinition,
};

mod element;
pub use element::{ParsedElement, ParsedTagClose, ParsedTagOpen};
