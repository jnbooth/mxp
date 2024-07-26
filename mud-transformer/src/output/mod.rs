mod buffer;
pub use buffer::BufferedOutput;

mod fragment;
pub use fragment::{EffectFragment, OutputDrain, OutputFragment, TelnetFragment, TextFragment};

mod output_trait;

mod span;
pub use span::{Heading, InList, TextFormat, TextStyle};
