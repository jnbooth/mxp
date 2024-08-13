use std::str;

use super::error::{Error, ErrorKind};

pub fn is_valid(target: &str) -> bool {
    let s: &[u8] = target.as_ref();
    !s.is_empty()
        && s[0].is_ascii_alphabetic()
        && s.iter()
            .all(|&c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'.')
}

pub fn validate(target: &str, error: ErrorKind) -> crate::Result<()> {
    if is_valid(target) {
        Ok(())
    } else {
        Err(Error::new(target, error))
    }
}
