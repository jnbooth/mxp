mod buffer;
pub(crate) use buffer::BufferedOutput;

mod color;
pub(crate) use color::TermColor;

mod fragment;
pub use fragment::{
    EffectFragment, EntityFragment, Output, OutputDrain, OutputFragment, TelnetFragment,
    TelnetSource, TelnetVerb, TextFragment, TextFragmentANSI, TextFragmentHtml,
};

mod span;
pub(crate) use span::EntitySetter;
pub use span::TextStyle;
