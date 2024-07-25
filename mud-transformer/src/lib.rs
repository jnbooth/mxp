#[macro_use]
extern crate enumeration;

mod escape;

mod receive;

mod output;
pub use output::{EffectFragment, OutputDrain, OutputFragment, TextFragment, TextStyle};

mod transformer;
pub use transformer::{Transformer, TransformerConfig};
