#[macro_use]
extern crate enumeration;

mod escape;

mod receive;
pub use receive::TelnetDelegate;

mod output;
pub use output::{EffectFragment, OutputDrain, OutputFragment, TextFragment, TextStyle};

mod transformer;
pub use transformer::{Transformer, TransformerConfig};
