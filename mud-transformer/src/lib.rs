#[macro_use]
extern crate enumeration;
#[macro_use]
extern crate enumeration_derive;

mod escape;

mod style;
pub use style::OutputFragment;

mod transformer;
pub use transformer::{Transformer, TransformerConfig};
