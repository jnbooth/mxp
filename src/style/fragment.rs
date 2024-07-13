use enumeration::EnumSet;

use super::span::{Heading, TextStyle};
use crate::color::WorldColor;
use crate::mxp::Link;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutputFragment {
    Hr,
    Image(String),
    Text(TextFragment),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextFragment {
    pub(super) text: Vec<u8>,
    pub(super) flags: EnumSet<TextStyle>,
    pub(super) foreground: WorldColor,
    pub(super) background: WorldColor,
    pub(super) action: Option<Link>,
    pub(super) heading: Option<Heading>,
    /// Which variable to set (FLAG in MXP).
    pub(super) variable: Option<String>,
}
