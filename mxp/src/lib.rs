#[macro_use]
mod macros;

mod argument;
pub use argument::Arguments;

mod collection;
pub use collection::{ElementComponent, State};

mod color;
pub use color::{HexOutOfRangeError, NamedColorIter, ParseHexColorError, RgbColor};

mod element;
pub use element::*;

mod entity;
pub use entity::{Entity, EntityEntry, EntityMap, PublishedIter};

pub mod escape;

mod keyword;
pub use keyword::{EntityKeyword, MxpKeyword};

mod lookup;

mod protocol;
pub use protocol::responses;

mod parser;
pub use parser::{validate, Error, ErrorKind, Words};

pub type Result<T> = std::result::Result<T, Error>;

pub const VERSION: &str = "0.5";

#[cfg(test)]
mod test_utils;
