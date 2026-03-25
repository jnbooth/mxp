use std::fmt;

use crate::arguments::{ArgumentScanner, ExpectArg, FromArgs};
use crate::color::RgbColor;

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
///     "<GAUGE Hp MAX=HpMax CAPTION=Health COLOR=red>".parse::<mxp::Gauge>(),
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

impl<'a, S: AsRef<str>> FromArgs<'a, S> for Gauge<S> {
    fn from_args<A: ArgumentScanner<'a, Decoded = S>>(mut scanner: A) -> crate::Result<Self> {
        let entity = scanner.get_next()?.expect_some("EntityName")?;
        let max = scanner.get_next_or("max")?;
        let caption = scanner.get_next_or("caption")?;
        let color = scanner.get_next_or("color")?.expect_color()?;
        scanner.expect_end()?;
        Ok(Self {
            entity,
            max,
            caption,
            color,
        })
    }
}

impl_from_str!(Gauge);

impl<S: AsRef<str>> fmt::Display for Gauge<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Gauge {
            entity,
            max,
            caption,
            color,
        } = self.borrow_text();
        crate::display::ElementFormatter {
            name: "GAUGE",
            arguments: &[&entity, &max, &caption, &color],
            keywords: &[],
        }
        .fmt(f)
    }
}
