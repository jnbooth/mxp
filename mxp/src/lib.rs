macro_rules! const_non_zero {
    ($i:ident, $t:ident, $n:expr) => {
        const $i: std::num::$t = match std::num::$t::new($n) {
            Some(n) => n,
            None => unreachable!(),
        };
    };
}

macro_rules! match_ci {
    (
        $s:expr,
        $l:literal $(| $lo:literal)* => $i:expr,
        $(_ => $default:expr)? $(,)?
    ) => {
        if $s.eq_ignore_ascii_case($l) $(|| $s.eq_ignore_ascii_case($lo))* {
            $i
        } $(else {
            $default
        })?
    };
    (
        $s:expr,
        $l_first:literal $(| $lo_first:literal)* => $i_first:expr,
        $($l:literal $(| $lo:literal)* => $i:expr),*
        $(, _ => $default:expr)? $(,)?
    ) => {
        if $s.eq_ignore_ascii_case($l_first) $(|| $s.eq_ignore_ascii_case($lo_first))* {
            $i_first
        } $(else if $s.eq_ignore_ascii_case($l) $(|| $s.eq_ignore_ascii_case($lo))* {
            $i
        })* $(else {
            $default
        })?
    };
}

mod argument;
pub use argument::Arguments;

mod collection;
pub use collection::{ElementComponent, State};

mod color;
pub use color::{HexOutOfRangeError, NamedColorIter, ParseHexColorError, RgbColor};

mod element;
pub use element::*;

mod entity;
pub use entity::{Entity, EntityEntry, EntityMap, PublishedIter};

pub mod escape;

mod keyword;
pub use keyword::{EntityKeyword, MxpKeyword};

mod lookup;

mod protocol;
pub use protocol::responses;

mod parser;
pub use parser::{validate, Error, ErrorKind, Words};

pub type Result<T> = std::result::Result<T, Error>;

pub const VERSION: &str = "0.5";

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod tests {
    #[test]
    fn match_ci_is_case_insensitive() {
        let result = match_ci! {"teSt",
            " TEST" => " TEST",
            "TEst" => "TEst",
            "teSt " => "teSt ",
            _ => "unmatched"
        };
        assert_eq!(result, "TEst");
    }
}
