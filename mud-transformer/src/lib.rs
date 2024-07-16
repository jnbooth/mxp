#[macro_use]
extern crate enumeration;

mod escape;

mod output;
pub use output::{OutputDrain, OutputFragment};

mod transformer;
pub use transformer::{SideEffect, Transformer, TransformerConfig};
