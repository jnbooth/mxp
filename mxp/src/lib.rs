#[macro_use]
extern crate enumeration;

mod color;
pub use color::{HexColor, WorldColor};

mod entity;
pub use entity::*;

mod protocol;
pub use protocol::responses;

pub const VERSION: &str = "0.5";
