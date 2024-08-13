mod error;
pub use error::{HexOutOfRangeError, ParseHexColorError};

mod named;

mod rgb;
pub use rgb::RgbColor;

#[cfg(feature = "serde")]
mod serde;

mod xterm;
