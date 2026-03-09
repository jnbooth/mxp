use std::str;

use super::error::{Error, ErrorKind};

fn is_valid(target: &str) -> bool {
    let s: &[u8] = target.as_ref();
    !s.is_empty()
        && s.first().is_some_and(u8::is_ascii_alphabetic)
        && s.iter()
            .all(|&c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'.')
}

/// If the specified target is valid to use as an MXP identifier or value, returns `Ok(())`.
/// Otherwise, returns an [`mxp::Error`](Error) for the target with the specified error kind.
///
/// # Examples
///
/// ```
/// let err = mxp::ErrorKind::InvalidEntityName;
/// assert!(mxp::validate("abc", err).is_ok());
/// assert!(mxp::validate("aBc_-.", err).is_ok());
/// assert!(mxp::validate("", err).is_err());
/// assert!(mxp::validate("_test", err).is_err());
/// assert!(mxp::validate("abc!", err).is_err());
/// ```
pub fn validate(target: &str, error: ErrorKind) -> crate::Result<()> {
    if is_valid(target) {
        Ok(())
    } else {
        Err(Error::new(target, error))
    }
}
