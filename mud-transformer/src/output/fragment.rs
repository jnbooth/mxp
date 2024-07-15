use std::fmt::{self, Display, Formatter};
use std::vec;

use bytes::Bytes;
use enumeration::EnumSet;

use super::span::{Heading, TextStyle};
use mxp::WorldColor;

pub type OutputDrain<'a> = vec::Drain<'a, OutputFragment>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutputFragment {
    Hr,
    Image(String),
    Text(TextFragment),
}

impl OutputFragment {
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            OutputFragment::Text(fragment) => Some(&fragment.text),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextFragment {
    pub(super) text: Bytes,
    pub(super) flags: EnumSet<TextStyle>,
    pub(super) foreground: WorldColor,
    pub(super) background: WorldColor,
    pub(super) action: Option<mxp::Link>,
    pub(super) heading: Option<Heading>,
    /// Which variable to set (FLAG in MXP).
    pub(super) variable: Option<String>,
}

impl AsRef<[u8]> for TextFragment {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.text
    }
}

impl Display for TextFragment {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        String::from_utf8_lossy(&self.text).fmt(f)
    }
}
