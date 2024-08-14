use std::borrow::Cow;
use std::str::FromStr;

use super::screen::{Align, Dimension};
use crate::argument::scan::{Decoder, Scan};
use crate::keyword::ImageKeyword;
use crate::parser::{Error, ErrorKind};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Image<S = String> {
    pub fname: Option<S>,
    pub url: Option<S>,
    pub class: Option<S>,
    pub height: Option<Dimension>,
    pub width: Option<Dimension>,
    pub hspace: Option<Dimension>,
    pub vspace: Option<Dimension>,
    pub align: Option<Align>,
    pub is_map: bool,
}

impl<'a> Image<&'a str> {
    pub fn into_owned(&self) -> Image {
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

impl<'a> Image<Cow<'a, str>> {
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

fn try_parse<S: AsRef<str>, T: FromStr>(
    output: crate::Result<Option<S>>,
) -> crate::Result<Option<T>> {
    let output = match output {
        Ok(Some(output)) => output,
        Ok(None) => return Ok(None),
        Err(e) => return Err(e),
    };
    let output = output.as_ref();
    match output.parse() {
        Ok(parsed) => Ok(Some(parsed)),
        Err(_) => Err(Error::new(output, ErrorKind::InvalidEntityNumber)),
    }
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for Image<D::Output<'a>> {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        Ok(Self {
            fname: scanner.next_or("fname")?,
            url: scanner.next_or("url")?,
            class: scanner.get("T")?,
            height: try_parse(scanner.get("H"))?,
            width: try_parse(scanner.get("W"))?,
            hspace: try_parse(scanner.get("HSPACE"))?,
            vspace: try_parse(scanner.get("VSPACE"))?,
            align: try_parse(scanner.get("ALIGN"))?,
            is_map: scanner.into_keywords().contains(ImageKeyword::IsMap),
        })
    }
}
