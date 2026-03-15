use super::error::{Error, ErrorKind};

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

/// Equivalent to [`str::from_utf8`], but returns an [`mxp::Error`](Error) instead of a
/// [`Utf8Error`](std::str::Utf8Error).
pub fn validate_utf8(bytes: &[u8]) -> crate::Result<&str> {
    if let Ok(utf8) = str::from_utf8(bytes) {
        return Ok(utf8);
    }
    Err(Error::new(
        String::from_utf8_lossy(bytes),
        ErrorKind::MalformedBytes,
    ))
}

/// If the specified target is valid to use as an MXP identifier or value, returns `true`.
/// Otherwise, returns `false`.
///
/// # Examples
///
/// ```
/// let err = mxp::ErrorKind::InvalidEntityName;
/// assert!(mxp::is_valid("abc"));
/// assert!(mxp::is_valid("aBc_-."));
/// assert!(!mxp::is_valid(""));
/// assert!(!mxp::is_valid("_test"));
/// assert!(!mxp::is_valid("abc!"));
/// ```
pub const fn is_valid(target: &str) -> bool {
    let [b'A'..=b'Z' | b'a'..=b'z', rest @ ..] = target.as_bytes() else {
        return false;
    };
    let mut i = 0;
    while i < rest.len() {
        if !matches!(rest[i], b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'-'| b'.') {
            return false;
        }
        i += 1;
    }
    true
}
