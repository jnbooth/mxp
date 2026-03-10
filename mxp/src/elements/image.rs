use std::borrow::Cow;

use crate::argument::{Decoder, ExpectArg as _, Scan};
use crate::keyword::ImageKeyword;
use crate::parser::Error;
use crate::screen::{Align, Dimension};

/// Displays an inline graphics image.
///
/// See [MXP specification: `<IMAGE>`](https://www.zuggsoft.com/zmud/mxp.htm#Images).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Image<S = String> {
    /// The name of the graphics file to display.
    pub fname: Option<S>,
    /// The URL of the path for the graphic if it should be downloaded on the fly.
    /// The classname is appended to the URL, along with the name of the graphics file itself.
    pub url: Option<S>,
    /// The class for the image.
    pub class: Option<S>,
    /// The height of the image. If omitted, the height is computed from the actual image.
    /// If the specified height is different from the image, the image is stretched.
    pub height: Option<Dimension<u32>>,
    /// The width of the image. If omitted, the width is computed from the actual image.
    /// If the specified width is different from the image, the image is stretched.
    pub width: Option<Dimension<u32>>,
    /// Additional space to add before and after the image.
    pub hspace: Option<Dimension<u32>>,
    /// Additional space to add above and below the image.
    pub vspace: Option<Dimension<u32>>,
    /// Controls the alignment of the image on the line.  For example, if ALIGN=Bottom is used, the rest of the text on the line will align with the bottom of the image.
    pub align: Option<Align>,
    /// Indicates that the image is an image-map. When an image-map is included within a `<SEND>`
    /// tag, the command sent to the MUD is appended with `"?X,Y"` where X,Y is the position clicked
    /// on the image.
    pub is_map: bool,
}

impl<S> Image<S> {
    pub fn map_text<T, F>(self, mut f: F) -> Image<T>
    where
        F: FnMut(S) -> T,
    {
        Image {
            fname: self.fname.map(&mut f),
            url: self.url.map(&mut f),
            class: self.class.map(f),
            height: self.height,
            width: self.width,
            hspace: self.hspace,
            vspace: self.vspace,
            align: self.align,
            is_map: self.is_map,
        }
    }
}

impl_into_owned!(Image);

impl<S: AsRef<str>> Image<S> {
    pub fn borrow_text(&self) -> Image<&str> {
        Image {
            fname: self.fname.as_ref().map(AsRef::as_ref),
            url: self.url.as_ref().map(AsRef::as_ref),
            class: self.class.as_ref().map(AsRef::as_ref),
            height: self.height,
            width: self.width,
            hspace: self.hspace,
            vspace: self.vspace,
            align: self.align,
            is_map: self.is_map,
        }
    }
}

impl_partial_eq!(Image);

impl<'a, D> TryFrom<Scan<'a, D>> for Image<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
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
                .and_then(|align| align.parse().ok()),
            is_map: scanner.into_keywords().contains(ImageKeyword::IsMap),
        })
    }
}
