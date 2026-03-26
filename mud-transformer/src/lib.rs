pub use bytes::Bytes;
pub use bytestring::ByteString;
pub use mxp;
pub use mxp::escape;

mod bytestring_ext;

mod input;
pub use input::InputDrain;

pub mod opt;
pub use opt::naws::subnegotiate as naws;

mod protocol;
pub use protocol::{TelnetSource, TelnetVerb};

pub mod output;

pub mod responses;

pub mod term;

mod transformer;
pub use transformer::{ByteSet, TabBehavior, Tag, Transformer, TransformerConfig, UseMxp};

fn count_bytes(haystack: &[u8], needle: u8) -> usize {
    haystack.iter().fold(0, |n, c| n + u32::from(*c == needle)) as usize
}
