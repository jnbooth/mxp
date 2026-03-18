//! Batteries-included implementation of [MXP (MUD eXtension Protocol)].
//!
//! [MXP (MUD eXtension Protocol)]: https://www.zuggsoft.com/zmud/mxp.htm
//!
//! # Description
//!
//! [MXP (MUD eXtension Protocol)] is an open communication protocol for MUD servers and clients.
//! The mxp library is a lightweight but robust implementation of the protocol in its entirety.
//! It is geared toward client implementations, but it can also be used for server-side syntax
//! handling.
//!
//! By default, mxp processes all tags described in the above MXP standard. To restrict which
//! elements your client supports, send a [`SupportResponse`] to the MUD server.
//!
//! [`SupportResponse`]: crate::responses::SupportResponse
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
//! Although useful for development, this approach lacks support for the most important aspect of
//! MXP: defining custom entities and elements. (It's also inefficient, because it uses owned
//! strings rather than borrowed string slices.) Instead, production environments should make use of
//! mxp's state management system, provided by [`mxp::State`] and [`mxp::ModeState`].
//!
//! [`mxp::State`]: State
//! [`mxp::ModeState`]: ModeState
//!
//! ## State management
//!
//! [`mxp::State`] stores definitions for custom [`Element`]s, [`Entity`]s, and [`LineTag`]s. Once
//! the state receives a definition, it can be used in all subsequent parses. With this approach,
//! rather than using [`FromStr`] to parse tags into owned strings, [`Tag::parse`] is used to
//! deserialize tags in-place using borrowed string slices.
//!
//! Unlike everything else in MXP, which uses XML syntax, line modes are set by ANSI escape
//! sequences. For example, to set the MXP mode to 20, the MUD server would send `<ESC>[20z`.
//! As such, it is up to the client to recognize MXP mode changes and apply them with
//! [`ModeState::set`] and [`ModeState::revert`].
//!
//! [`FromStr`]: std::str::FromStr
//! [`Tag::parse`]: node::Tag::parse
//!
//! ```
//! use std::borrow::Cow;
//! // Alternatively:
//! // - `use mxp::node;` for prefixed names, e.g. `node::TagOpen`
//! // - `use mxp::node::{Tag as TagNode, TagOpen as TagOpenNode};`
//! use mxp::node::{Tag, TagOpen};
//!
//! // Handler function for receiving tags from the MUD server. A tag is anything surrounded by
//! // `<` and `>`. The `secure` flag indicates whether the current line mode is secure.
//! fn handle_tag(mxp_state: &mut mxp::State, mut src: &str, secure: bool) -> mxp::Result<()> {
//!     src = &src[1..src.len() - 1]; // strip < and > from the source
//!     match Tag::parse(src, secure)? {
//!         Tag::Definition(definition) => { // <!...>
//!             mxp_state.define(definition)?;
//!         }
//!         Tag::Open(tag) => { // <...>
//!             handle_open(tag, mxp_state, secure)?; // see below
//!         }
//!         Tag::Close(tag) => (), // </...>
//!     }
//!     Ok(())
//! }
//!
//!
//! // Handler function for receiving an opening tag. Called by `handle_tag`.
//! fn handle_open(tag: TagOpen, mxp_state: &mxp::State, secure: bool) -> mxp::Result<()> {
//!     match mxp_state.get_component(tag.name, secure)? {
//!         // server sent a standard, atomic tag, such as <a> or <br>
//!         mxp::Component::AtomicTag(atom) => {
//!             let action = atom.decode(&tag.arguments, mxp_state)?;
//!             handle_action(&action);
//!         }
//!         // server sent a previously-defined custom element
//!         mxp::Component::Element(el) => {
//!             for action in el.decode(&tag.arguments, mxp_state) {
//!                 handle_action(&action?);
//!             }
//!         }
//!     }
//!     Ok(())
//! }
//!
//! // Handler function for applying the action of an atomic tag.
//! // This is where the actual client logic takes place.
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
//! // initialize state
//! let mut mxp_state = mxp::State::with_globals();
//! let mut mode = mxp::ModeState::new();
//!
//! mode.set(mxp::Mode::SECURE_ONCE);
//! let secure = mode.use_secure();
//! assert!(secure); // line mode is secure, so elements may be defined
//! handle_tag(&mut mxp_state, "<!ELEMENT MyEl '<HR><BR>' EMPTY OPEN>", secure).unwrap();
//! // The <MyEl> element has now been defined, and can be used in subsequent parses.
//!
//! let secure = mode.use_secure();
//! assert!(!secure); // SECURE_ONCE reverts back to OPEN mode after use
//! handle_tag(&mut mxp_state, "<myel>", secure).unwrap(); // prints "----\n" (<HR><BR>)
//! ```
//!
//! ## Server-side usage
//!
//! All of the types exported by mxp can be serialized to MXP syntax with their [`Display`]
//! implementation.
//!
//! [`Display`]: std::fmt::Display
//!
//! ```
//! use mxp::entity::EntityKeyword;
//!
//! let entity = mxp::node::EntityDefinition {
//!     name: "Guilds",
//!     value: "Wizards",
//!     desc: None,
//!     keywords: EntityKeyword::Publish | EntityKeyword::Add,
//! };
//! assert_eq!(entity.to_string(), "<!EN Guilds \"Wizards\" PUBLISH ADD>");
//! ```
//!
//! For advanced tag building, see [`TagBuilder`].
//!
//! [`TagBuilder`]: node::TagBuilder
//!
//! # Memory allocation
//!
//! [`Tag::parse`] allocates memory if it parses a custom element definition (as
//! [`node::ElementDefinition`]), which needs to use owned strings because custom elements are
//! stored long-term in state. Otherwise, it only allocates memory to parse arguments passed to
//! an opening tag (as [`node::TagOpen`]), as described in the next paragraph.
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
//! [`Cow`]: std::borrow::Cow

#[macro_use]
mod case_insensitive;

pub mod arguments;
pub use arguments::Arguments;

mod case_fold_map;
pub(crate) use case_fold_map::CaseFoldMap;

pub mod color;
pub use color::RgbColor;

pub(crate) mod display;

pub mod element;
pub use element::{Action, ActionKind, AtomicTag, Element, ParseAs};

pub mod elements;
pub use elements::*;

pub mod entity;
pub use entity::Entity;

pub mod escape;

mod keyword;

mod line;
pub use line::{LineTag, LineTagProperties, Mode, ModeRangeError, ModeState};

mod parse;
pub use parse::{Decoder, Error, ErrorKind, is_valid, validate, validate_utf8};

pub mod node;

pub mod responses;

mod screen;
pub use screen::{Align, Dimension, DimensionUnit};

mod state;
pub use state::{Component, State};

/// Type alias for `Result<T, mxp::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Reexport from the [`flagset`](flagset::FlagSet) package.
pub use flagset::FlagSet;

#[cfg(test)]
mod test_utils;
