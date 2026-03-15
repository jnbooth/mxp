use std::borrow::Cow;
use std::str::FromStr;

use crate::arguments::ExpectArg as _;
use crate::keyword::ImageKeyword;
use crate::parse::{Decoder, Scan};
use crate::screen::{Align, Dimension};

/// Displays an inline graphics image.
///
/// Clients typically treat images as large individual characters.  Images always take up the
/// entire height of the line; text cannot wrap lines next to an image. Text wrapping to the next
/// line will wrap at the normal left margin of the screen until the graphics image.
///
/// See [MXP specification: `<IMAGE>`](https://www.zuggsoft.com/zmud/mxp.htm#Images).
///
/// # Examples
///
/// ```
/// use mxp::{Align, Dimension};
///
/// assert_eq!(
///     "<IMAGE map.jpg URL='http://example.org:5000/images' T=combat H=1c W=6c HSPACE=10 VSPACE=5 ALIGN=left ISMAP>".parse::<mxp::Image>(),
///     Ok(mxp::Image {
///         fname: "map.jpg".into(),
///         url: Some("http://example.org:5000/images".into()),
///         class: Some("combat".into()),
///         height: Some(Dimension::character_spacing(1)),
///         width: Some(Dimension::character_spacing(6)),
///         hspace: Some(Dimension::pixels(10)),
///         vspace: Some(Dimension::pixels(5)),
///         align: Some(Align::Left),
///         is_map: true,
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Image<S = String> {
    /// The name of the graphics file to display.
    pub fname: S,
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
    /// Controls the alignment of the image on the line. For example, if `ALIGN=Bottom` is used, the
    /// rest of the text on the line will align with the bottom of the image.
    pub align: Option<Align>,
    /// Indicates that the image is an image-map. When an image-map is included within a `<SEND>`
    /// tag, the command sent to the MUD is appended with `"?X,Y"` where X,Y is the position clicked
    /// on the image.
    pub is_map: bool,
}

impl<S> Image<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, mut f: F) -> Image<T>
    where
        F: FnMut(S) -> T,
    {
        Image {
            fname: f(self.fname),
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
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Image<&str> {
        Image {
            fname: self.fname.as_ref(),
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

    /// Combines `self.url`, `self.class`, and `self.fname` into a single URI.
    pub fn uri(&self) -> Cow<'_, str> {
        let fname = self.fname.as_ref();
        if self.url.is_none() && self.class.is_none() {
            return Cow::Borrowed(fname);
        }
        let url = match &self.url {
            Some(url) => url.as_ref(),
            None => "",
        };
        let class = match &self.class {
            Some(class) => class.as_ref(),
            None => "",
        };
        let mut buf = String::with_capacity(url.len() + class.len() + fname.len() + 2);
        for part in [url, class] {
            if part.is_empty() {
                continue;
            }
            buf.push_str(part);
            if !part.ends_with('/') {
                buf.push('/');
            }
        }
        buf.push_str(fname);
        Cow::Owned(buf)
    }
}

impl_partial_eq!(Image);

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Image<Cow<'a, str>> {
    type Error = crate::Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        let fname = scanner.next_or("fname")?.expect_some("fname")?;
        let url = scanner.next_or("url")?;
        let class = scanner.next_or("t")?;
        let height = scanner.next_or("h")?.expect_number()?;
        let width = scanner.next_or("w")?.expect_number()?;
        let hspace = scanner.next_or("hspace")?.expect_number()?;
        let vspace = scanner.next_or("vspace")?.expect_number()?;
        let align = scanner.next_or("align")?.expect_variant()?;
        let keywords = scanner.into_keywords()?;
        let is_map = keywords.contains(ImageKeyword::IsMap);
        Ok(Self {
            fname,
            url,
            class,
            height,
            width,
            hspace,
            vspace,
            align,
            is_map,
        })
    }
}

impl FromStr for Image {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Image)
    }
}
