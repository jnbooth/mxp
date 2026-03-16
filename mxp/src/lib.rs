//! Batteries-included implementation of [MXP (MUD eXtension Protocol)].
//!
//! [MXP (MUD eXtension Protocol)]: https://www.zuggsoft.com/zmud/mxp.htm
//!
//! # Description
//!
//! [MXP (MUD eXtension Protocol)] is an open communication protocol for MUD servers and clients.
//! The mxp library is a lightweight yet robust implementation of the entire protocol.
//!
//! # Examples
//!
//! ## Simple parsing
//!
//! ```
//! use mxp::{Dimension, FrameAction, FrameLayout};
//!
//! assert_eq!(
//!     "<FRAME NAME=Map LEFT=-20c TOP=0 WIDTH=20c HEIGHT=20c>".parse::<mxp::Frame>(),
//!     Ok(mxp::Frame {
//!         name: "Map".into(),
//!         action: FrameAction::Open,
//!         title: "Map".into(),
//!         scrolling: false,
//!         layout: FrameLayout::External {
//!             left: Dimension::character_spacing(-20),
//!             top: Dimension::pixels(0),
//!             width: Some(Dimension::character_spacing(20)),
//!             height: Some(Dimension::character_spacing(20)),
//!             floating: false,
//!         },
//!     }),
//! );
//! ```
//!
//! ## State management
//!
//! ```
//! use std::borrow::Cow;
//! use mxp::parsed::{ParsedElement, ParsedTagClose, ParsedTagOpen};
//!
//! fn handle_element(mxp_state: &mut mxp::State, mut source: &str, secure: bool) -> mxp::Result<()> {
//!     source = &source[1..source.len() - 1]; // remove < and >
//!     match ParsedElement::parse(source, secure)? {
//!         ParsedElement::Definition(definition) => {
//!             mxp_state.define(definition)?;
//!         }
//!         ParsedElement::TagOpen(tag) => {
//!             handle_open(tag, mxp_state, secure)?;
//!         }
//!         ParsedElement::TagClose(tag) => (),
//!     }
//!     Ok(())
//! }
//!
//! fn handle_open(tag: ParsedTagOpen, mxp_state: &mxp::State, secure: bool) -> mxp::Result<()> {
//!     let component = mxp_state.get_component(tag.name, secure)?;
//!     let args = tag.arguments.parse_args()?;
//!     match component {
//!         mxp::Component::Atom(atom) => {
//!             let action = atom.decode(&args, mxp_state)?;
//!             handle_action(&action);
//!         }
//!         mxp::Component::Element(el) => {
//!             for action in el.decode(&args, mxp_state) {
//!                 handle_action(&action?);
//!             }
//!         }
//!     }
//!     Ok(())
//! }
//!
//! fn handle_action(action: &mxp::Action<Cow<str>>) {
//!     use mxp::Action;
//!
//!     match action {
//!         Action::Br => println!(),
//!         Action::Hr => print!("----"),
//!         _ => (),
//!     }
//! }
//!
//! let mut mxp_state = mxp::State::with_globals();
//! let mut mode = mxp::ModeState::new();
//! mode.set(mxp::Mode::SECURE_ONCE);
//! let secure = mode.use_secure(); // true
//! handle_element(&mut mxp_state, "<!ELEMENT custom '<HR><BR>'> EMPTY OPEN>", secure).unwrap();
//! let secure = mode.use_secure(); // false – SECURE_ONCE reverts back to OPEN mode after use
//! handle_element(&mut mxp_state, "<custom>", secure).unwrap(); // prints "----\n"
//! ```

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
