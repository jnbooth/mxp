#[macro_use]
extern crate enumeration;

pub use mxp;
pub use mxp::escape;

mod receive;

mod output;
pub use output::{
    EffectFragment, OutputDrain, OutputFragment, TelnetFragment, TextFragment, TextStyle,
};

mod transformer;
pub use transformer::{Transformer, TransformerConfig};
