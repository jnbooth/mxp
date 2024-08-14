mod buffer;
pub use buffer::BufferedOutput;

mod color;
pub use color::TermColor;

mod fragment;
pub use fragment::{
    EffectFragment, Output, OutputDrain, OutputFragment, TelnetFragment, TextFragment,
};

mod shared_string;
pub use shared_string::SharedString;

mod span;
pub use span::{InList, TextStyle};
