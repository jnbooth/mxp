use std::borrow::Cow;

use super::screen::{Align, Dimension};
use crate::argument::{Decoder, ExpectArg, Scan};
use crate::keyword::ImageKeyword;
use crate::parser::Error;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Image<S = String> {
    pub fname: Option<S>,
    pub url: Option<S>,
    pub class: Option<S>,
    pub height: Option<Dimension<u32>>,
    pub width: Option<Dimension<u32>>,
    pub hspace: Option<Dimension<u32>>,
    pub vspace: Option<Dimension<u32>>,
    pub align: Option<Align>,
    pub is_map: bool,
}

impl Image<&str> {
    pub fn into_owned(self) -> Image {
        Image {
            fname: self.fname.map(ToOwned::to_owned),
            url: self.url.map(ToOwned::to_owned),
            class: self.class.map(ToOwned::to_owned),
            height: self.height,
            width: self.width,
            hspace: self.hspace,
            vspace: self.vspace,
            align: self.align,
            is_map: self.is_map,
        }
    }
}

impl Image<Cow<'_, str>> {
    pub fn into_owned(self) -> Image {
        Image {
            fname: self.fname.map(Cow::into_owned),
            url: self.url.map(Cow::into_owned),
            class: self.class.map(Cow::into_owned),
            height: self.height,
            width: self.width,
            hspace: self.hspace,
            vspace: self.vspace,
            align: self.align,
            is_map: self.is_map,
        }
    }
}

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Image<Cow<'a, str>> {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        Ok(Self {
            fname: scanner.next_or("fname")?,
            url: scanner.next_or("url")?,
            class: scanner.next_or("T")?,
            height: scanner.next_or("H")?.expect_number()?,
            width: scanner.next_or("W")?.expect_number()?,
            hspace: scanner.next_or("HSPACE")?.expect_number()?,
            vspace: scanner.next_or("VSPACE")?.expect_number()?,
            align: scanner
                .next_or("ALIGN")?
                .and_then(|align| align.as_ref().parse().ok()),
            is_map: scanner.into_keywords().contains(ImageKeyword::IsMap),
        })
    }
}
