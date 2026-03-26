use std::str::Utf8Error;

use bytes::Bytes;
use bytestring::ByteString;
use bytestringmut::ByteStringMut;

pub trait ByteStringExt: Sized {
    fn from_utf8(bytes: Bytes) -> Result<Self, Utf8Error>;
}

impl ByteStringExt for ByteString {
    fn from_utf8(bytes: Bytes) -> Result<Self, Utf8Error> {
        str::from_utf8(&bytes)?;
        // SAFETY: bytes are valid UTF-8.
        Ok(unsafe { ByteString::from_bytes_unchecked(bytes) })
    }
}

pub trait ByteStringMutExt {
    fn share(&mut self, s: &str) -> ByteString;
}

impl ByteStringMutExt for ByteStringMut {
    fn share(&mut self, s: &str) -> ByteString {
        self.clear();
        self.push_str(s);
        self.split().freeze()
    }
}
