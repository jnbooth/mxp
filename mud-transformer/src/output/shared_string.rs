use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;
use std::str;

use bytes::Bytes;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SharedString {
    inner: Bytes,
}

impl Default for SharedString {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedString {
    pub const fn new() -> Self {
        Self {
            inner: Bytes::new(),
        }
    }

    /// Converts a `Bytes` to a string slice without checking
    /// that the string contains valid UTF-8.
    ///
    /// # Safety
    ///
    /// The bytes passed in must be valid UTF-8.
    #[inline]
    pub unsafe fn from_utf8_unchecked(utf8: Bytes) -> Self {
        Self { inner: utf8 }
    }

    #[inline]
    fn as_str(&self) -> &str {
        // SAFETY: Held bytes are valid UTF-8.
        unsafe { str::from_utf8_unchecked(&self.inner) }
    }

    #[inline]
    pub fn into_bytes(self) -> Bytes {
        self.inner
    }
}

impl AsRef<str> for SharedString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for SharedString {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for SharedString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for SharedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.as_str())
    }
}

impl From<SharedString> for Bytes {
    fn from(value: SharedString) -> Self {
        value.inner
    }
}

impl From<&SharedString> for String {
    fn from(value: &SharedString) -> Self {
        value.as_str().to_owned()
    }
}
