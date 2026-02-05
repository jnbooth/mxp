#[macro_use]
mod macros;

mod argument;
pub use argument::Arguments;

mod collection;
pub use collection::{DecodeElement, ElementComponent, State};

mod color;
pub use color::{HexOutOfRangeError, NamedColorIter, ParseHexColorError, RgbColor};

mod element;
pub use element::*;

mod entity;
pub use entity::{DecodedEntity, Entity, EntityEntry, EntityInfo, EntityMap, PublishedIter};

pub mod escape;

mod keyword;
pub use keyword::{EntityKeyword, MxpKeyword};

mod parser;
pub use parser::{Error, ErrorKind, ParseErrorTarget, Words, validate};

pub mod responses;

pub type Result<T> = std::result::Result<T, Error>;

pub use flagset::FlagSet;

pub const VERSION: &str = "0.5";

#[cfg(test)]
mod test_utils;
