use std::borrow::Cow;
use std::str::FromStr;

use crate::parse::{Decoder, Error, ExpectArg as _, Scan};

/// Displays an MXP entity value as status bar text.
///
/// See [MXP specification: `<STAT>`](https://www.zuggsoft.com/zmud/mxp.htm#Using%20Entities).
///
/// # Examples
///
/// ```
/// use mxp::RgbColor;
///
/// assert_eq!(
///     "<Stat Hp MAX=HpMax Caption=Health>".parse::<mxp::Stat>(),
///     Ok(mxp::Stat {
///         entity: "Hp".into(),
///         max: Some("HpMax".into()),
///         caption: Some("Health".into()),
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Stat<S = String> {
    /// Name of the entity to use as text data.
    pub entity: S,
    /// Name of the entity to use for the maximum value of the data.
    pub max: Option<S>,
    /// Optional caption text.
    pub caption: Option<S>,
}

impl<S> Stat<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, mut f: F) -> Stat<T>
    where
        F: FnMut(S) -> T,
    {
        Stat {
            entity: f(self.entity),
            max: self.max.map(&mut f),
            caption: self.caption.map(f),
        }
    }
}

impl_into_owned!(Stat);

impl<S: AsRef<str>> Stat<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Stat<&str> {
        Stat {
            entity: self.entity.as_ref(),
            max: self.max.as_ref().map(AsRef::as_ref),
            caption: self.caption.as_ref().map(AsRef::as_ref),
        }
    }
}

impl_partial_eq!(Stat);

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Stat<Cow<'a, str>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let entity = scanner.next()?.expect_some("EntityName")?;
        let max = scanner.next_or("max")?;
        let caption = scanner.next_or("caption")?;
        scanner.expect_end()?;
        Ok(Self {
            entity,
            max,
            caption,
        })
    }
}

impl FromStr for Stat {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Stat)
    }
}
