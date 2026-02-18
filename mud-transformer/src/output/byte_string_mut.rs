use std::ops::Deref;

use bytes::BytesMut;
use bytestring::ByteString;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteStringMut {
    inner: BytesMut,
}

impl ByteStringMut {
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn push_str(&mut self, s: &str) {
        self.inner.extend_from_slice(s.as_bytes());
    }

    pub fn split(&mut self) -> Self {
        Self {
            inner: self.inner.split(),
        }
    }

    pub fn freeze(self) -> ByteString {
        let bytes = self.inner.freeze();
        // SAFETY: `bytes` contains only valid UTF-8.
        unsafe { ByteString::from_bytes_unchecked(bytes) }
    }

    pub fn as_str(&self) -> &str {
        // SAFETY: `self.inner` contains only valid UTF-8.
        unsafe { str::from_utf8_unchecked(&self.inner) }
    }
}

impl Deref for ByteStringMut {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for ByteStringMut {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for ByteStringMut {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}
