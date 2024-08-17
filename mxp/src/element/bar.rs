use std::borrow::Cow;

use crate::argument::{Decoder, ExpectArg, Scan};
use crate::color::RgbColor;
use crate::parser::Error;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Gauge<S = String> {
    pub entity: S,
    pub max: Option<S>,
    pub caption: Option<S>,
    pub color: Option<RgbColor>,
}

impl Gauge<&str> {
    pub fn into_owned(self) -> Gauge {
        Gauge {
            entity: self.entity.to_owned(),
            max: self.max.map(ToOwned::to_owned),
            caption: self.caption.map(ToOwned::to_owned),
            color: self.color,
        }
    }
}

impl<'a> Gauge<Cow<'a, str>> {
    pub fn into_owned(self) -> Gauge {
        Gauge {
            entity: self.entity.into_owned(),
            max: self.max.map(Cow::into_owned),
            caption: self.caption.map(Cow::into_owned),
            color: self.color,
        }
    }
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for Gauge<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            entity: scanner.next()?.expect_arg("EntityName")?,
            max: scanner.next_or("max")?,
            caption: scanner.next_or("caption")?,
            color: scanner
                .next_or("color")?
                .and_then(|color| RgbColor::named(color.as_ref())),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stat<S = String> {
    pub entity: S,
    pub max: Option<S>,
    pub caption: Option<S>,
}

impl Stat<&str> {
    pub fn into_owned(self) -> Stat {
        Stat {
            entity: self.entity.to_owned(),
            max: self.max.map(ToOwned::to_owned),
            caption: self.caption.map(ToOwned::to_owned),
        }
    }
}

impl<'a> Stat<Cow<'a, str>> {
    pub fn into_owned(self) -> Stat {
        Stat {
            entity: self.entity.into_owned(),
            max: self.max.map(Cow::into_owned),
            caption: self.caption.map(Cow::into_owned),
        }
    }
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for Stat<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            entity: scanner.next()?.expect_arg("EntityName")?,
            max: scanner.next_or("max")?,
            caption: scanner.next_or("caption")?,
        })
    }
}
