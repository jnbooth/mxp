#[macro_use]
extern crate enumeration;

pub use mxp;
pub use mxp::escape;

macro_rules! const_non_zero {
    ($i:ident, $t:ident, $n:expr) => {
        const $i: std::num::$t = match std::num::$t::new($n) {
            Some(n) => n,
            None => unreachable!(),
        };
    };
}

mod receive;

mod output;
pub use output::{
    EffectFragment, EntityFragment, Output, OutputDrain, OutputFragment, SharedString,
    TelnetFragment, TextFragment, TextStyle,
};

mod transformer;
pub use transformer::{InputDrain, Transformer, TransformerConfig, UseMxp};
