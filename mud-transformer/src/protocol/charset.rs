use std::error::Error;
use std::io::Write;
use std::iter::FusedIterator;
use std::{fmt, io};

use flagset::{FlagSet, flags};
use mxp::escape::telnet;

use super::negotiate::{Negotiate, write_escaping_iac};
use crate::count_bytes;
use crate::transformer::TransformerConfig;

/// Negotiate About Character Set
pub const OPT: u8 = 42;

pub const REQUEST: u8 = 1;
pub const ACCEPTED: u8 = 2;
pub const REJECTED: u8 = 3;
pub const TTABLE_IS: u8 = 4;
pub const TTABLE_REJECTED: u8 = 5;
pub const TTABLE_ACK: u8 = 6;
pub const TTABLE_NAK: u8 = 7;

/// Default separator used internally
const SEP: u8 = b' ';

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    EmptyString,
    NoVersion,
    UnsupportedVersion(u8),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyString => f.write_str("received an empty string"),
            Self::NoVersion => f.write_str("string terminated early"),
            Self::UnsupportedVersion(ver) => write!(f, "expected version 1, got {ver}"),
        }
    }
}

impl Error for DecodeError {}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Request<'a> {
    pub ttable: bool,
    pub sep: u8,
    pub charsets: &'a [u8],
}

impl<'a> Request<'a> {
    pub fn decode(request: &'a [u8]) -> Result<Self, DecodeError> {
        let [sep, rest @ ..] = request else {
            return Err(DecodeError::EmptyString);
        };
        let (ttable, rest) = match rest.strip_prefix(b"TTABLE") {
            Some([b' ', rest @ ..] | rest) => (true, rest),
            None => (false, rest),
        };
        let rest = match rest {
            [1 | b'1', rest @ ..] => rest,
            [version, ..] => return Err(DecodeError::UnsupportedVersion(*version)),
            [] => return Err(DecodeError::NoVersion),
        };
        Ok(Self {
            ttable,
            sep: *sep,
            charsets: rest,
        })
    }

    pub fn iter(self) -> Iter<'a> {
        self.into_iter()
    }
}

impl<'a> IntoIterator for Request<'a> {
    type Item = &'a [u8];

    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            sep: self.sep,
            charsets: self.charsets,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Iter<'a> {
    sep: u8,
    charsets: &'a [u8],
}

impl<'a> Iter<'a> {
    fn finish(&mut self) -> Option<&'a [u8]> {
        if self.charsets.is_empty() {
            return None;
        }
        let charset = self.charsets;
        self.charsets = &[];
        Some(charset)
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let Some(pos) = self.charsets.iter().position(|&c| c == self.sep) else {
            return self.finish();
        };
        let charset = &self.charsets[..pos];
        self.charsets = &self.charsets[pos + 1..];
        Some(charset)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.len();
        (exact, Some(exact))
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let Some(pos) = self.charsets.iter().rposition(|&c| c == self.sep) else {
            return self.finish();
        };
        let charset = &self.charsets[pos + 1..];
        self.charsets = &self.charsets[..pos];
        Some(charset)
    }
}

impl ExactSizeIterator for Iter<'_> {
    fn len(&self) -> usize {
        if self.charsets.is_empty() {
            0
        } else {
            1 + count_bytes(self.charsets, self.sep)
        }
    }
}

impl FusedIterator for Iter<'_> {}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TTableEntry<'a> {
    /// Sequence of 7-BIT ASCII printable characters which identifies the character set.
    pub charset: &'a [u8],
    /// Number of bits nominally required for each character in the corresponding table.
    /// Should be a multiple of 8.
    pub size: u8,
    /// How many characters (of the maximum `2**<char size>`) are being transmitted in the
    /// corresponding map.
    /// Should be no greater than `0xFFFFFF`. Values exceeding that range will be truncated.
    pub count: u32,
    /// Corresponding `<char count>` number of characters. These characters form a mapping from all
    /// or part of the characters in one of the specified character sets to the correct characters
    /// in the other character set.
    pub map: &'a [u8],
}

pub trait ToBeBytes: Copy {
    type BeBytes;

    fn to_be_bytes(self) -> Self::BeBytes;
}

impl<C: ToBeBytes> ToBeBytes for &C {
    type BeBytes = C::BeBytes;

    fn to_be_bytes(self) -> Self::BeBytes {
        (*self).to_be_bytes()
    }
}

macro_rules! impl_to_be_bytes {
    ($t:ty, $n:literal) => {
        impl ToBeBytes for $t {
            type BeBytes = [u8; $n];

            fn to_be_bytes(self) -> Self::BeBytes {
                self.to_be_bytes()
            }
        }
    };
}

impl_to_be_bytes!(u8, 1);
impl_to_be_bytes!(u16, 2);
impl_to_be_bytes!(u32, 4);
impl_to_be_bytes!(u64, 8);

impl ToBeBytes for char {
    type BeBytes = [u8; 4];

    fn to_be_bytes(self) -> Self::BeBytes {
        u32::from(self).to_be_bytes()
    }
}

impl<'a> TTableEntry<'a> {
    pub fn new<S, const N: usize, I>(charset: &'a S, chars: I, buf: &'a mut Vec<u8>) -> Self
    where
        S: AsRef<[u8]>,
        I: IntoIterator,
        I::Item: ToBeBytes<BeBytes = [u8; N]>,
    {
        let initial_len = buf.len();
        let chars = chars.into_iter();
        buf.reserve(chars.size_hint().0 * N);
        for ch in chars {
            buf.extend_from_slice(&ch.to_be_bytes());
        }
        let total_size = buf.len() - initial_len;
        #[allow(clippy::cast_possible_truncation)]
        Self {
            charset: charset.as_ref(),
            size: (N * 8) as u8,
            count: (total_size / N) as u32,
            map: &buf[initial_len..],
        }
    }

    pub fn write_metadata_to<W: Write>(&self, mut writer: W, sep: u8) -> io::Result<()> {
        writer.write_all(self.charset)?;
        let [_, c1, c2, c3] = self.count.to_be_bytes();
        writer.write_all(&[sep, self.size, c1, c2, c3])
    }

    pub fn write_map_to<W: Write>(&self, writer: W) -> io::Result<()> {
        write_escaping_iac(writer, self.map)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Response<'a> {
    /// The receiver is already sending text to and expecting text from the sender to be encoded in
    /// one of the specified character sets.
    Agree,
    /// The receiver is capable of handling at least one of the specified character sets.
    Accept(&'a [u8]),
    /// The receiver of the CHARSET REQUEST message acknowledges its receipt and is transmitting a pair of tables which define the mapping between specified character sets.
    TTable(TTableEntry<'a>, TTableEntry<'a>),
    /// The receiver of the CHARSET REQUEST message acknowledges its receipt but refuses to use any of the requested character sets.
    Reject,
}

impl Response<'_> {
    pub fn write_to<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_all(&[telnet::IAC, telnet::SB, OPT])?;
        match self {
            Self::Agree => writer.write_all(&[ACCEPTED])?,
            Self::Accept(charset) => {
                writer.write_all(&[ACCEPTED])?;
                writer.write_all(charset)?;
            }
            Self::TTable(a, b) => {
                writer.write_all(&[TTABLE_IS, 1, SEP])?;
                a.write_metadata_to(&mut writer, SEP)?;
                writer.write_all(&[SEP])?;
                b.write_metadata_to(&mut writer, SEP)?;
                a.write_map_to(&mut writer)?;
                b.write_map_to(&mut writer)?;
            }
            Self::Reject => writer.write_all(&[REJECTED])?,
        }
        writer.write_all(&[telnet::IAC, telnet::SB])
    }
}

flags! {
    enum Charset: u8 {
        Ascii,
        Utf8
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Charsets {
    inner: FlagSet<Charset>,
}

impl TryFrom<&[u8]> for Charsets {
    type Error = DecodeError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let request = Request::decode(data)?;
        let mut flags = FlagSet::default();
        for fragment in request {
            if fragment.eq_ignore_ascii_case(b"UTF-8") {
                flags |= Charset::Utf8;
            } else if fragment.eq_ignore_ascii_case(b"US-ASCII") {
                flags |= Charset::Ascii;
            }
        }
        Ok(Charsets { inner: flags })
    }
}

impl TryFrom<&str> for Charsets {
    type Error = DecodeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.as_bytes())
    }
}

impl Charsets {
    pub const fn new() -> Self {
        Self {
            inner: FlagSet::empty(),
        }
    }
}

impl Default for Charsets {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Charsets {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.inner.contains(Charset::Utf8) {
            f.write_str("\x02UTF-8")
        } else if self.inner.contains(Charset::Ascii) {
            f.write_str("\x02US-ASCII")
        } else {
            f.write_str("\x03")
        }
    }
}

impl Negotiate for Charsets {
    const OPT: u8 = OPT;

    fn negotiate<W: fmt::Write>(mut self, mut f: W, config: &TransformerConfig) -> fmt::Result {
        if config.disable_utf8 {
            self.inner -= Charset::Utf8;
        }
        write!(f, "{self}")
    }
}
