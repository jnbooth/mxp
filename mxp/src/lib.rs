#[macro_use]
mod case_insensitive;

mod argument;
pub use argument::{Arguments, KeywordFilter, NoKeywords};

mod collections;
pub use collections::{Component, State};

mod color;
pub use color::{HexOutOfRangeError, NamedColorIter, ParseHexColorError, RgbColor};

mod element;
pub use element::*;

mod entity;
pub use entity::{
    DecodedEntity, Entity, EntityEntry, EntityInfo, EntityMap, EntityVisibility, PublishedIter,
};

pub mod escape;

mod keyword;
pub use keyword::{EntityKeyword, MxpKeyword};

mod parser;
pub use parser::{Error, ErrorKind, ParseErrorTarget, Words, is_valid, validate};

pub mod responses;

pub type Result<T> = std::result::Result<T, Error>;

pub use flagset::FlagSet;

#[cfg(test)]
mod test_utils;
