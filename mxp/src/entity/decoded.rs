use std::borrow::Cow;
use std::fmt;

/// A decoded MXP entity, either standard or server-defined.
///
/// See [MXP specification: Entities](https://www.zuggsoft.com/zmud/mxp.htm#ENTITY).
#[derive(Clone, PartialEq, Eq)]
pub enum DecodedEntity<'a> {
    /// A standard XML entity, such as `&amp;` or `&#032;`.
    Standard(char),
    /// A custom MXP entity defined by the server.
    Custom(Cow<'a, str>),
}

impl Default for DecodedEntity<'_> {
    fn default() -> Self {
        Self::Custom(Cow::Borrowed(""))
    }
}

impl<'a> DecodedEntity<'a> {
    /// Appends the decoded entity to a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use mxp::entity::DecodedEntity;
    ///
    /// let mut buf = String::new();
    /// DecodedEntity::Standard('>').push_to(&mut buf);
    /// DecodedEntity::Custom("Warrior".into()).push_to(&mut buf);
    /// assert_eq!(buf, ">Warrior");
    /// ```
    ///
    /// However, [`write!`] is usually more ergonomic:
    ///
    /// ```
    /// use std::fmt::Write;
    /// use mxp::entity::DecodedEntity;
    ///
    /// # fn test() -> std::fmt::Result {
    /// let mut buf = String::new();
    /// write!(buf, "{}{}", DecodedEntity::Standard('>'), DecodedEntity::Custom("Warrior".into()))?;
    /// assert_eq!(buf, ">Warrior");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn push_to(self, buf: &mut String) {
        match self {
            Self::Standard(c) => buf.push(c),
            Self::Custom(s) => buf.push_str(&s),
        }
    }

    /// If this entity is a string ([`Self::Custom`]), the entity is returned directly.
    /// If this entity is a character ([`Self::Standard`]), encodes the character as UTF-8 into the
    /// provided byte buffer, and then returns the subslice of the buffer that contains the encoded
    /// character.
    ///
    /// See [`char::encode_utf8`].
    #[inline]
    pub fn encode(&'a self, buf: &'a mut [u8]) -> &'a str {
        match self {
            Self::Standard(c) => c.encode_utf8(buf),
            Self::Custom(s) => s,
        }
    }
}

impl From<char> for DecodedEntity<'_> {
    #[inline]
    fn from(value: char) -> Self {
        Self::Standard(value)
    }
}

impl<'a> From<&'a str> for DecodedEntity<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::Custom(value.into())
    }
}

impl<'a> From<Cow<'a, str>> for DecodedEntity<'a> {
    #[inline]
    fn from(value: Cow<'a, str>) -> Self {
        Self::Custom(value)
    }
}

impl From<String> for DecodedEntity<'_> {
    #[inline]
    fn from(value: String) -> Self {
        Self::Custom(value.into())
    }
}

impl fmt::Debug for DecodedEntity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Standard(c) => c.fmt(f),
            Self::Custom(s) => s.fmt(f),
        }
    }
}

impl fmt::Display for DecodedEntity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Standard(c) => c.fmt(f),
            Self::Custom(s) => s.fmt(f),
        }
    }
}

impl PartialEq<char> for DecodedEntity<'_> {
    fn eq(&self, other: &char) -> bool {
        match self {
            Self::Standard(ch) => ch == other,
            Self::Custom(_) => false,
        }
    }
}

impl<'a> PartialEq<DecodedEntity<'a>> for char {
    fn eq(&self, other: &DecodedEntity<'a>) -> bool {
        *other == *self
    }
}

impl PartialEq<str> for DecodedEntity<'_> {
    fn eq(&self, other: &str) -> bool {
        match self {
            Self::Standard(_) => false,
            Self::Custom(s) => **s == *other,
        }
    }
}

impl<'a> PartialEq<DecodedEntity<'a>> for str {
    fn eq(&self, other: &DecodedEntity<'a>) -> bool {
        *other == *self
    }
}
