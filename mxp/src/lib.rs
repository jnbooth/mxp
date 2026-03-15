#[macro_use]
mod case_insensitive;

pub mod arguments;
pub use arguments::Arguments;

mod case_fold_map;
pub(crate) use case_fold_map::CaseFoldMap;

pub mod color;
pub use color::RgbColor;

pub mod element;
pub use element::{Action, ActionKind, Element, Tag};

pub mod elements;
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

mod state;
pub use state::{Component, State};

pub type Result<T> = std::result::Result<T, Error>;

/// Reexport from the [`flagset`](flagset::FlagSet) package.
pub use flagset::FlagSet;

#[cfg(test)]
mod test_utils;
