use std::str;

use super::error::{Error as MxpError, ParseError};

pub fn is_valid(target: &str) -> bool {
    let s: &[u8] = target.as_ref();
    !s.is_empty()
        && s[0].is_ascii_alphabetic()
        && s.iter()
            .all(|&c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'.')
}

pub fn validate(target: &str, error: MxpError) -> Result<(), ParseError> {
    if is_valid(target) {
        Ok(())
    } else {
        Err(ParseError::new(target, error))
    }
}
