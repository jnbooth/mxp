#[macro_use]
extern crate enumeration;

#[macro_use]
extern crate enumeration_derive;

mod color;
pub use color::{HexColor, WorldColor};

mod entity;
pub use entity::*;

mod protocol;
pub use protocol::responses;

pub const VERSION: &str = "0.5";
