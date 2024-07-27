use std::borrow::Borrow;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::str;

use bytes::Bytes;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SharedString {
    inner: Bytes,
}

impl SharedString {
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

impl Display for SharedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl From<SharedString> for Bytes {
    fn from(value: SharedString) -> Self {
        value.inner
    }
}
