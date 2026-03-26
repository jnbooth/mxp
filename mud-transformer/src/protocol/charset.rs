use std::io::Write;
use std::str;
use std::{fmt, io};

use flagset::{FlagSet, flags};
use mxp::escape::telnet;

use super::negotiate::{Negotiate, write_escaping_iac};
use crate::transformer::TransformerConfig;

/// [RFC 2066](https://datatracker.ietf.org/doc/html/rfc2066): CHARSET
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
    InvalidCharacter(u8),
    NoCharsets,
    NoVersion,
    UnsupportedVersion(u8),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidCharacter(c) => write!(f, "received invalid character {c:#X}"),
            Self::NoCharsets => f.write_str("charset list is empty"),
            Self::NoVersion => f.write_str("string terminated early"),
            Self::UnsupportedVersion(ver) => write!(f, "expected version 1, got {ver}"),
        }
    }
}

/// `IAC SB CHARSET REQUEST <...> IAC SE`
///
/// The sender requests that all text sent to and by it be encoded in one of the specified
/// character sets.
#[derive(Clone, Debug)]
pub struct Request<'a> {
    /// The sender is willing to accept a mapping (translation table) between any character set
    /// listed in [`charsets`](Self::charsets) and any character set desired by the receiver.
    pub ttable: bool,
    /// Sequence of 7-bit ASCII printable characters, consisting of one or more character sets.
    /// The character sets should appear in order of preference (most preferred first).
    pub charsets: str::Split<'a, char>,
}

impl<'a> Request<'a> {
    /// Decodes a request from a subnegotiation, i.e. `IAC SB CHARSET REQUEST <request> IAC SE`.
    pub fn decode(request: &'a [u8]) -> Result<Self, DecodeError> {
        let (ttable, rest) = match request.strip_prefix(b"TTABLE") {
            Some([b' ', rest @ ..] | rest) => (true, rest),
            None => (false, request),
        };
        let rest = match rest {
            [] => return Err(DecodeError::NoVersion),
            [1 | b'1', rest @ ..] => rest,
            [version, ..] => return Err(DecodeError::UnsupportedVersion(*version)),
        };
        let (sep, rest) = match rest {
            [] | [_] => return Err(DecodeError::NoCharsets),
            [sep, rest @ ..] => (*sep, rest),
        };
        if !sep.is_ascii() {
            return Err(DecodeError::InvalidCharacter(sep));
        }
        let charsets = str::from_utf8(rest)
            .map_err(|e| DecodeError::InvalidCharacter(rest[e.valid_up_to()]))?;
        Ok(Self {
            ttable,
            charsets: charsets.split(sep as char),
        })
    }
}

impl<'a> IntoIterator for Request<'a> {
    type Item = &'a str;

    type IntoIter = str::Split<'a, char>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.charsets
    }
}

/// Translation table for mapping  between character sets.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TTable<'a> {
    /// Sequence of 7-BIT ASCII printable characters which identifies the character set.
    pub charset: &'a str,
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

/// Types that can be converted to byte arrays in big-endian order.
pub trait ToBeBytes: Copy {
    /// Resulting array type. Should be `[u8; N]`.
    type BeBytes;

    /// Converts to a big-endian byte array.
    fn to_be_bytes(self) -> Self::BeBytes;
}

impl<C: ToBeBytes> ToBeBytes for &C {
    type BeBytes = C::BeBytes;

    #[inline]
    fn to_be_bytes(self) -> Self::BeBytes {
        (*self).to_be_bytes()
    }
}

macro_rules! impl_to_be_bytes {
    ($t:ty, $n:literal) => {
        impl ToBeBytes for $t {
            type BeBytes = [u8; $n];

            #[inline]
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

    #[inline]
    fn to_be_bytes(self) -> Self::BeBytes {
        u32::from(self).to_be_bytes()
    }
}

impl<'a> TTable<'a> {
    /// Constructs a `TTable` for the specified character set, using `buf` as the backing buffer.
    pub fn new<S, const N: usize, I>(charset: &'a str, buf: &'a mut Vec<u8>, chars: I) -> Self
    where
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
            charset,
            size: (N * 8) as u8,
            count: (total_size / N) as u32,
            map: &buf[initial_len..],
        }
    }

    fn encode_metadata<W: Write>(&self, mut writer: W, sep: u8) -> io::Result<()> {
        writer.write_all(self.charset.as_bytes())?;
        let [_, c1, c2, c3] = self.count.to_be_bytes();
        write_escaping_iac(writer, &[sep, self.size, c1, c2, c3])
    }

    fn encode_map<W: Write>(&self, writer: W) -> io::Result<()> {
        write_escaping_iac(writer, self.map)
    }
}

/// Response to a [`Request`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Response<'a> {
    /// `IAC SB CHARSET ACCEPTED IAC SE`
    ///
    /// The receiver is already sending text to and expecting text from the sender to be encoded in
    /// one of the specified character sets.
    Agreed,
    /// `IAC SB CHARSET ACCEPTED <charset> IAC SE`
    ///
    /// The receiver is capable of handling at least one of the specified character sets.
    Accepted(&'a str),
    /// `IAC SB CHARSET TTABLE-IS <...> IAC SE`
    ///
    /// The receiver of the CHARSET REQUEST message acknowledges its receipt and is transmitting a
    /// pair of tables which define the mapping between specified character sets.
    TTableIs(TTable<'a>, TTable<'a>),
    /// `IAC SB CHARSET REJECTED IAC SE`
    ///
    /// The receiver of the CHARSET REQUEST message acknowledges its receipt but refuses to use any
    /// of the requested character sets.
    Rejected,
}

impl Response<'_> {
    /// Writes the subnegotiation data to a writer, including IAC prefix and suffix.
    pub fn encode<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_all(&[telnet::IAC, telnet::SB, OPT])?;
        match self {
            Self::Agreed => writer.write_all(&[ACCEPTED])?,
            Self::Accepted(charset) => {
                writer.write_all(&[ACCEPTED])?;
                writer.write_all(charset.as_bytes())?;
            }
            Self::TTableIs(a, b) => {
                writer.write_all(&[TTABLE_IS, 1, SEP])?;
                a.encode_metadata(&mut writer, SEP)?;
                writer.write_all(&[SEP])?;
                b.encode_metadata(&mut writer, SEP)?;
                a.encode_map(&mut writer)?;
                b.encode_map(&mut writer)?;
            }
            Self::Rejected => writer.write_all(&[REJECTED])?,
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
            if fragment.eq_ignore_ascii_case("UTF-8") {
                flags |= Charset::Utf8;
            } else if fragment.eq_ignore_ascii_case("US-ASCII") {
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
