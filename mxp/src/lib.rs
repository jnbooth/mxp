#[macro_use]
extern crate enumeration;

mod color;
pub use color::{ansi16, HexColor, WorldColor};

mod entity;
pub use entity::*;

pub mod escape;

mod protocol;
pub use protocol::responses;

pub const VERSION: &str = "0.5";
