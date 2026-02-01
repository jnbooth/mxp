use std::fmt;
use std::num::NonZero;

use bytestring::ByteString;
use flagset::FlagSet;
use mxp::{Heading, RgbColor};

use super::OutputFragment;
use crate::output::TextStyle;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextFragment {
    pub text: ByteString,
    pub flags: FlagSet<TextStyle>,
    pub foreground: Option<RgbColor>,
    pub background: Option<RgbColor>,
    pub font: Option<ByteString>,
    pub size: Option<NonZero<u8>>,
    pub action: Option<mxp::Link>,
    pub heading: Option<mxp::Heading>,
}

impl From<TextFragment> for OutputFragment {
    fn from(value: TextFragment) -> Self {
        Self::Text(value)
    }
}

impl TextFragment {
    pub fn ansi(&self) -> TextFragmentANSI<'_> {
        TextFragmentANSI { fragment: self }
    }

    pub fn html(&self) -> TextFragmentHtml<'_> {
        TextFragmentHtml { fragment: self }
    }
}

impl fmt::Display for TextFragment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.text.fmt(f)
    }
}

impl From<&str> for TextFragment {
    fn from(value: &str) -> Self {
        Self {
            text: value.into(),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct TextFragmentANSI<'a> {
    fragment: &'a TextFragment,
}

#[derive(Copy, Clone)]
struct Escaping(pub bool);

impl Escaping {
    pub const fn prefix(&mut self) -> &'static str {
        if self.0 {
            return ";";
        }
        self.0 = true;
        "\x1B["
    }
}

impl fmt::Display for TextFragmentANSI<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let frag = self.fragment;
        let mut escaping = Escaping(false);
        let mut flags = frag.flags;
        if frag.action.is_some() {
            flags |= TextStyle::Underline;
        }
        for ansi in flags.into_iter().filter_map(TextStyle::ansi) {
            write!(f, "{}{ansi}", escaping.prefix())?;
        }
        if let Some(RgbColor { r, g, b }) = frag.foreground {
            write!(f, "{}38;2;{r};{g};{b}", escaping.prefix())?;
        }
        if let Some(RgbColor { r, g, b }) = frag.background {
            write!(f, "{}48;2;{r};{g};{b}", escaping.prefix())?;
        }
        if escaping.0 {
            write!(f, "m{}\x1B[0m", frag.text)
        } else {
            write!(f, "{}", frag.text)
        }
    }
}

#[derive(Debug)]
pub struct TextFragmentHtml<'a> {
    fragment: &'a TextFragment,
}

impl fmt::Display for TextFragmentHtml<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct StyleSeparator(bool);
        impl StyleSeparator {
            pub fn write(&mut self, f: &mut fmt::Formatter) -> fmt::Result {
                if self.0 {
                    return f.write_str(";");
                }
                self.0 = true;
                f.write_str(" style=\"")
            }
        }

        let mut sep = StyleSeparator(false);
        let frag = self.fragment;
        let text = html_escape::encode_text(&frag.text);
        if let Some(action) = &frag.action {
            write!(f, "<a href=\"{}\">", action.action)?;
        }
        let tag = match frag.heading {
            Some(Heading::H1) => "h1",
            Some(Heading::H2) => "h2",
            Some(Heading::H3) => "h3",
            Some(Heading::H4) => "h4",
            Some(Heading::H5) => "h5",
            Some(Heading::H6) => "h6",
            None => "span",
        };
        write!(f, "<{tag}")?;
        if frag.flags.contains(TextStyle::Bold) {
            sep.write(f)?;
            f.write_str("font-weight:bold")?;
        }
        if frag.flags.contains(TextStyle::Italic) {
            sep.write(f)?;
            f.write_str("font-style:italic")?;
        }
        if frag.flags.contains(TextStyle::Underline) {
            sep.write(f)?;
            f.write_str("text-decoration:underline")?;
        }
        if frag.flags.contains(TextStyle::Strikeout) {
            sep.write(f)?;
            f.write_str("text-decoration:line-through")?;
        }
        if let Some(fg) = frag.foreground {
            sep.write(f)?;
            write!(f, "color:#{fg:X}")?;
        }
        if let Some(bg) = frag.background
            && bg != RgbColor::BLACK
        {
            sep.write(f)?;
            write!(f, "background-color:#{bg:X}")?;
        }
        if let Some(font) = &frag.font {
            sep.write(f)?;
            write!(f, "font-family:{font}")?;
        }
        if let Some(size) = frag.size {
            sep.write(f)?;
            write!(f, "font-size:{size}px")?;
        }
        if sep.0 {
            f.write_str("\"")?;
        }
        write!(f, ">{text}</{tag}>")?;
        if frag.action.is_some() {
            write!(f, "</a>")?;
        }
        Ok(())
    }
}
