#[macro_use]
extern crate enumeration;

mod escape;

mod output;
pub use output::{EffectFragment, OutputDrain, OutputFragment, TextFragment, TextStyle};

mod transformer;
pub use transformer::{SideEffect, Transformer, TransformerConfig};
