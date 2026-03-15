#[macro_use]
mod case_insensitive;

mod collections;
pub use collections::{Component, State};

mod color;
pub use color::{HexOutOfRangeError, NamedColorIter, ParseHexColorError, RgbColor};

mod element;
pub use element::{Action, ActionKind, DecodeElement, Element, ElementItem, ParseAs, Tag};

mod elements;
pub use elements::*;

mod entity;
pub use entity::{
    DecodedEntity, Entity, EntityEntry, EntityInfo, EntityMap, EntityVisibility, PublishedIter,
};

pub mod escape;

mod keyword;
pub use keyword::{EntityKeyword, KeywordFilter};

mod line;
pub use line::{LineTag, LineTagProperties, Mode, ModeRangeError, ModeState};

mod parse;
pub use parse::{Arguments, Error, ErrorKind, is_valid, validate, validate_utf8};

pub mod parsed;

pub mod responses;

mod screen;
pub use screen::{Align, Dimension, DimensionUnit};

pub type Result<T> = std::result::Result<T, Error>;

pub use flagset::FlagSet;

#[cfg(test)]
mod test_utils;
