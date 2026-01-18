use std::borrow::Borrow;
use std::fmt;
use std::hash::Hash;
use std::ops::Deref;
use std::str;

use bytes::{Bytes, BytesMut};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BytesPool {
    inner: BytesMut,
}

impl BytesPool {
    pub fn share(&mut self, bytes: &[u8]) -> Bytes {
        self.inner.extend_from_slice(bytes);
        self.inner.split().freeze()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StringPool {
    inner: BytesMut,
}

impl StringPool {
    pub fn share(&mut self, s: &str) -> SharedString {
        self.inner.extend_from_slice(s.as_bytes());
        SharedString {
            inner: self.inner.split().freeze(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
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

impl AsRef<[u8]> for SharedString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
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

impl Hash for SharedString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl fmt::Display for SharedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Debug for SharedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
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

impl From<&str> for SharedString {
    fn from(value: &str) -> Self {
        Self {
            inner: Bytes::copy_from_slice(value.as_bytes()),
        }
    }
}

impl From<&String> for SharedString {
    fn from(value: &String) -> Self {
        value.as_str().into()
    }
}
