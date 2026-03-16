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
//! mxp can be used to parse MXP strings directly:
//!
//! ```
//! use mxp::{Dimension, FrameAction, FrameLayout};
//!
//! assert_eq!(
//!     "<FRAME NAME=Map LEFT=-20c TOP=0 WIDTH=20c HEIGHT=20c>".parse::<mxp::Action>(),
//!     Ok(mxp::Action::Frame(mxp::Frame {
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
//!     })),
//! );
//! ```
//!
//! However, this approach lacks the most important aspect of MXP parsing: custom entities and
//! elements. It also has no way to differentiate between secure modes and open modes.
//! (It's also inefficient, because it uses owned strings rather than borrowed string slices.)
//! The intended way to use this library is with state management via [`mxp::State`] and
//! [`mxp::ModeState`].
//!
//! [`mxp::State`]: State
//! [`mxp::ModeState`]: ModeState
//!
//! ## State management
//!
//! mxp provides state management via [`mxp::State`]: the central hub of MXP logic.
//! `mxp::State` stores custom [`Element`]s, custom [`Entity`]s, and user-defined [`LineTag`]s.
//! In this approach, rather than using [`FromStr`] to parse tags with owned strings,
//! [`ParsedElement::parse`] is used to deserialize tags in-place using borrowed string slices.
//!
//! Furthermore, [`mxp::ModeState`] can be used to handle differentiation between open and closed
//! modes, as well as retrieving custom elements from user-defined line tags. Rather than being
//! parsed from XML tags like everything else in MXP, modes are set by ANSI escape sequences. For
//! example, to set the MXP mode to 20, a server would send `<ESC>[20z`. As such, it is up to the
//! client to  recognize MXP mode changes and apply them with [`ModeState::update`] (or one of its
//! convenience aliases, such as [`ModeState::set`]).
//!
//! [`FromStr`]: std::str::FromStr
//! [`ParsedElement::parse`]: parsed::ParsedElement::parse
//!
//! ```
//! use std::borrow::Cow;
//! use mxp::parsed::{ParsedElement, ParsedTagClose, ParsedTagOpen};
//!
//! fn handle_element(mxp_state: &mut mxp::State, mut src: &str, secure: bool) -> mxp::Result<()> {
//!     src = &src[1..src.len() - 1]; // remove < and >
//!     match ParsedElement::parse(src, secure)? {
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
//!     match component {
//!         mxp::Component::AtomicTag(atom) => {
//!             let action = atom.decode(&tag.arguments, mxp_state)?;
//!             handle_action(&action);
//!         }
//!         mxp::Component::Element(el) => {
//!             for action in el.decode(&tag.arguments, mxp_state) {
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
//!
//! # Memory allocation
//!
//! [`ParsedElement::parse`] allocates memory if it parses a custom element definition (as
//! [`ParsedElementDefinition`]), which needs to use owned strings because custom elements are
//! stored long-term in state. Otherwise, it only allocates memory to parse arguments passed to
//! an opening tag (as [`ParsedTagOpen`]), as described in the next paragraph.
//!
//! [`Arguments`] parsing allocates a `Vec<&'a str>` and `HashMap<&'a str, &'a str>` for positional
//! and named arguments respectively. Both use generous size guesses (based on the number of spaces
//! in the string) in order to prevent reallocations. [`Arguments`] are ephemeral structs that drop
//! as soon as they are done being used to decode tags.
//!
//! Tag decoding (via [`AtomicTag::decode`] and [`Element::decode`]) uses [`Cow`]s because
//! attributes may contain entities, in which case they must be decoded to owned strings in order to
//! replace entities with their definitions (e.g. replacing `"&lt;"` with `"<"`). If the MXP string
//! does not contain entities, no allocations are performed.
//!
//! [`ParsedElementDefinition`]: parsed::ParsedElementDefinition
//! [`ParsedTagOpen`]: parsed::ParsedTagOpen
//! [`Cow`]: std::borrow::Cow

#[macro_use]
mod case_insensitive;

pub mod arguments;
pub use arguments::Arguments;

mod case_fold_map;
pub(crate) use case_fold_map::CaseFoldMap;

pub mod color;
pub use color::RgbColor;

pub mod element;
pub use element::{Action, ActionKind, AtomicTag, Element};

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
