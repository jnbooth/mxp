#[macro_use]
extern crate enumeration;
#[macro_use]
extern crate enumeration_derive;

mod color;

mod escape;

mod mxp;

mod style;

mod transformer;
pub use transformer::{Transformer, TransformerConfig};
