#[macro_use]
extern crate enumeration;
#[macro_use]
extern crate enumeration_derive;

mod adapter;
pub use adapter::*;

mod color;

mod escape;

mod mxp;

mod style;
pub use style::OutputFragment;

mod transformer;
pub use transformer::{Transformer, TransformerConfig};
