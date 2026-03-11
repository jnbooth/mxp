#[macro_use]
mod case_insensitive;

mod collections;
pub use collections::{Component, State};

mod color;
pub use color::{HexOutOfRangeError, NamedColorIter, ParseHexColorError, RgbColor};

mod element;
pub use element::{
    Action, ActionKind, CollectedDefinition, CollectedElement, DecodeElement, DefinitionKind,
    Element, ElementCommand, ElementItem, ParseAs, Tag,
};

mod elements;
pub use elements::*;

mod entity;
pub use entity::{
    DecodedEntity, Entity, EntityEntry, EntityInfo, EntityMap, EntityVisibility, PublishedIter,
};

pub mod escape;

mod keyword;
pub use keyword::{DestKeyword, EntityKeyword, KeywordFilter, MxpKeyword};

mod mode;
pub use mode::{Mode, ModeRangeError, ModeState};

mod parse;
pub use parse::{Arguments, Error, ErrorKind, ParseErrorTarget, Words, is_valid, validate};

pub mod responses;

mod screen;
pub use screen::{Align, Dimension, DimensionUnit};

pub type Result<T> = std::result::Result<T, Error>;

pub use flagset::FlagSet;

#[cfg(test)]
mod test_utils;
