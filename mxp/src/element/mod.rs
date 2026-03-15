//! [`Element`] and associated types.

mod action;
pub use action::Action;

mod action_kind;
pub use action_kind::ActionKind;

mod decoder;
pub use decoder::ElementDecoder;

mod element;
pub use element::Element;

mod item;
pub use item::ElementItem;

mod parse_as;
pub use parse_as::ParseAs;

mod tag;
pub use tag::Tag;
