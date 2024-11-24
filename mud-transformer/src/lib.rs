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

pub mod protocol;
pub use protocol::naws::subnegotiate as naws;

mod output;
pub use output::{
    EffectFragment, EntityFragment, Output, OutputDrain, OutputFragment, SharedString,
    TelnetFragment, TelnetSource, TelnetVerb, TextFragment, TextStyle,
};

mod transformer;
pub use transformer::{InputDrain, Tag, Transformer, TransformerConfig, UseMxp};
