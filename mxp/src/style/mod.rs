mod buffer;
pub use buffer::BufferedOutput;

mod fragment;
pub use fragment::OutputFragment;

mod output;

mod span;
pub use span::{Heading, InList, TextFormat, TextStyle};
