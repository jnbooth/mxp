pub use bytes::Bytes;
pub use bytestring::ByteString;

mod input;
pub use input::Drain as InputDrain;
pub use mxp;
pub use mxp::escape;

pub mod protocol;
pub use protocol::naws::subnegotiate as naws;

mod output;
pub use output::{
    ControlFragment, EntityFragment, MxpFragment, Output, OutputDrain, OutputFragment,
    TelnetFragment, TelnetSource, TelnetVerb, TextFragment, TextFragmentANSI, TextFragmentHtml,
    TextStyle,
};

pub mod responses;

pub mod term;

mod transformer;
pub use transformer::{TabBehavior, Tag, Transformer, TransformerConfig, UseMxp};
