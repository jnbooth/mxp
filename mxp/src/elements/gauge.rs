use std::borrow::Cow;

use crate::argument::{Decoder, ExpectArg as _, Scan};
use crate::color::RgbColor;
use crate::parser::Error;

/// Displays an MXP entity value as a gauge.
///
/// See [MXP specification: `<GAUGE>`](https://www.zuggsoft.com/zmud/mxp.htm#Using%20Entities).
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

impl Gauge<&str> {
    pub fn into_owned(self) -> Gauge<String> {
        Gauge {
            entity: self.entity.to_owned(),
            max: self.max.map(ToOwned::to_owned),
            caption: self.caption.map(ToOwned::to_owned),
            color: self.color,
        }
    }
}

impl Gauge<Cow<'_, str>> {
    pub fn into_owned(self) -> Gauge<String> {
        Gauge {
            entity: self.entity.into_owned(),
            max: self.max.map(Cow::into_owned),
            caption: self.caption.map(Cow::into_owned),
            color: self.color,
        }
    }
}

impl<'a, D> TryFrom<Scan<'a, D>> for Gauge<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            entity: scanner.next()?.expect_some("EntityName")?,
            max: scanner.next_or("max")?,
            caption: scanner.next_or("caption")?,
            color: scanner
                .next_or("color")?
                .and_then(|color| RgbColor::named(&color)),
        })
    }
}
