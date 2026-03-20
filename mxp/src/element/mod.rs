//! [`Element`] and associated types.

mod action;
pub use action::Action;

mod action_kind;
pub use action_kind::ActionKind;

mod atomic_tag;
pub use atomic_tag::AtomicTag;

mod attribute_list;
pub use attribute_list::AttributeList;

mod decoder;
pub use decoder::{ElementDecodeIter, ElementDecoder};

mod element;
pub use element::Element;

mod flag;
pub use flag::ElementFlag;

mod item;
pub use item::ElementItem;

mod parse_as;
pub use parse_as::ParseAs;
