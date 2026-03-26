mod buffer;
pub(crate) use buffer::BufferedOutput;

mod fragment;
pub use fragment::{
    ControlFragment, EntityFragment, MapperFragment, MxpFragment, Output, OutputDrain,
    OutputFragment, TelnetFragment, TextFragment, TextFragmentANSI, TextFragmentHtml,
    VariableFragment,
};

mod interpret_ansi;
pub use interpret_ansi::interpret_ansi;

mod link;
pub use link::{Link, SendTo};

mod register;

mod span;
pub use span::TextStyle;
