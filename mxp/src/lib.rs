#[macro_use]
extern crate enumeration;

macro_rules! match_ci {
    (
        $s:expr,
        $l_first:literal $(| $lo_first:literal)* => $i_first:expr,
        $($l:literal $(| $lo:literal)* => $i:expr),*
        $(, _ => $default:expr)? $(,)?
    ) => {
        if $s.eq_ignore_ascii_case($l_first) $(|| $s.eq_ignore_ascii_case($lo_first))* {
            $i_first
        }
        $(
            else if $s.eq_ignore_ascii_case($l) $(|| $s.eq_ignore_ascii_case($lo))* {
                $i
            }
        )*
        $(else {
            $default
        })?
    };
}

mod argument;
pub use argument::{ArgumentIndex, Arguments, FgColor, FontEffect, FontStyle, Keyword, XchMode};

mod collection;
pub use collection::{ElementComponent, ElementMap, EntityMap, State};

mod color;
pub use color::{HexOutOfRangeError, ParseHexColorError, RgbColor, TermColor};

mod entity;
pub use entity::*;

pub mod escape;

mod lookup;

mod protocol;
pub use protocol::responses;

mod parser;
pub use parser::{is_valid, validate, Error, ParseError, Words};

pub const VERSION: &str = "0.5";
