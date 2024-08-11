mod error;
pub use error::{HexOutOfRangeError, ParseHexColorError};

mod named;

mod rgb;
pub use rgb::RgbColor;

#[cfg(feature = "serde")]
mod serde;

mod term_color;
pub use term_color::TermColor;

mod xterm;
