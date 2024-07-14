mod argument;
pub use argument::{ArgumentIndex, Arguments, Keyword};

mod atom;
pub use atom::{Action, Atom, Tag, TagFlag};

mod element;
pub use element::{Element, ElementComponent, ElementMap};

mod entity_map;
pub use entity_map::EntityMap;

mod link;
pub use link::{Link, SendTo};

mod mode;
pub use mode::Mode;

mod validation;
pub use validation::{is_valid, validate, MxpError as Error, ParseError};

mod words;
pub use words::Words;

pub const VERSION: &str = "0.5";
