use std::borrow::Cow;

use crate::parser::{Error, ErrorKind};

pub const CHARS: &str = "\x20\x21\x22\x23\x24\x25\x26\x27\x28\x29\x2a\x2b\x2c\x2d\x2e\x2f\x30\x31\x32\x33\x34\x35\x36\x37\x38\x39\x3a\x3b\x3c\x3d\x3e\x3f\x40\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x4f\x50\x51\x52\x53\x54\x55\x56\x57\x58\x59\x5a\x5b\x5c\x5d\x5e\x5f\x60\x61\x62\x63\x64\x65\x66\x67\x68\x69\x6a\x6b\x6c\x6d\x6e\x6f\x70\x71\x72\x73\x74\x75\x76\x77\x78\x79\x7a\x7b\x7c\x7d\x7e\x7f";

pub fn decode_amps<'a, F>(mut s: &str, mut f: F) -> crate::Result<Cow<str>>
where
    F: FnMut(&str) -> crate::Result<Option<&'a str>>,
{
    let mut res = String::new();
    while let Some(start) = s.find('&') {
        if start > 0 {
            res.push_str(&s[..start]);
        }
        s = &s[start..];
        let end = s
            .find(';')
            .ok_or_else(|| Error::new(s, ErrorKind::NoClosingSemicolon))?;
        res.push_str(f(&s[1..end])?.unwrap_or(&s[..=end]));
        s = &s[end + 1..];
    }
    if res.is_empty() {
        return Ok(Cow::Borrowed(s));
    }
    if !s.is_empty() {
        res.push_str(s);
    }
    Ok(Cow::Owned(res))
}
