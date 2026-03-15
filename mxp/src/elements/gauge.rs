use std::borrow::Cow;
use std::str::FromStr;

use crate::arguments::ExpectArg as _;
use crate::color::RgbColor;
use crate::parse::{Decoder, Scan};

/// Displays an MXP entity value as a gauge.
///
/// See [MXP specification: `<GAUGE>`](https://www.zuggsoft.com/zmud/mxp.htm#Using%20Entities).
///
/// # Examples
///
/// ```
/// use mxp::RgbColor;
///
/// assert_eq!(
///     "<GAUGE Hp MAX=HpMax Caption=Health Color=red>".parse::<mxp::Gauge>(),
///     Ok(mxp::Gauge {
///         entity: "Hp".into(),
///         max: Some("HpMax".into()),
///         caption: Some("Health".into()),
///         color: Some(RgbColor::hex(0xFF0000)),
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Gauge<S = String> {
    /// Name of the entity to use as gauge data.
    pub entity: S,
    /// Name of the entity to use for the maximum value of the gauge.
    pub max: Option<S>,
    /// Optional text to display on the gauge.
    pub caption: Option<S>,
    /// Color of the gauge bar.
    pub color: Option<RgbColor>,
}

impl<S> Gauge<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, mut f: F) -> Gauge<T>
    where
        F: FnMut(S) -> T,
    {
        Gauge {
            entity: f(self.entity),
            max: self.max.map(&mut f),
            caption: self.caption.map(f),
            color: self.color,
        }
    }
}

impl_into_owned!(Gauge);

impl<S: AsRef<str>> Gauge<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Gauge<&str> {
        Gauge {
            entity: self.entity.as_ref(),
            max: self.max.as_ref().map(AsRef::as_ref),
            caption: self.caption.as_ref().map(AsRef::as_ref),
            color: self.color,
        }
    }
}

impl_partial_eq!(Gauge);

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Gauge<Cow<'a, str>> {
    type Error = crate::Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let entity = scanner.next()?.expect_some("EntityName")?;
        let max = scanner.next_or("max")?;
        let caption = scanner.next_or("caption")?;
        let color = scanner.next_or("color")?.expect_color()?;
        scanner.expect_end()?;
        Ok(Self {
            entity,
            max,
            caption,
            color,
        })
    }
}

impl FromStr for Gauge {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Gauge)
    }
}
