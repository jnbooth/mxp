#[macro_use]
mod case_insensitive;

pub mod arguments;
pub use arguments::Arguments;

mod collections;
pub use collections::{Component, State};

pub mod color;
pub use color::RgbColor;

mod element;
pub use element::{Action, ActionKind, DecodeElement, Element, ElementItem, ParseAs, Tag};

mod elements;
pub use elements::*;

pub mod entity;
pub use entity::Entity;

pub mod escape;

mod keyword;
pub use keyword::KeywordFilter;

mod line;
pub use line::{LineTag, LineTagProperties, Mode, ModeRangeError, ModeState};

mod parse;
pub use parse::{Error, ErrorKind, is_valid, validate, validate_utf8};

pub mod parsed;

pub mod responses;

mod screen;
pub use screen::{Align, Dimension, DimensionUnit};

pub type Result<T> = std::result::Result<T, Error>;

pub use flagset::FlagSet;

#[cfg(test)]
mod test_utils;
