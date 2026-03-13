mod buffer;
pub(crate) use buffer::BufferedOutput;

mod fragment;
pub use fragment::{
    ControlFragment, EntityFragment, MxpFragment, Output, OutputDrain, OutputFragment,
    TelnetFragment, TextFragment, TextFragmentANSI, TextFragmentHtml, VariableFragment,
};

mod interpret_ansi;
pub use interpret_ansi::interpret_ansi;

mod link;
pub use link::{Link, SendTo};

mod span;
pub(super) use span::SpanList;
pub use span::TextStyle;
