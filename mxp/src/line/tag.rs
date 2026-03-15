use std::ops::Deref;

use crate::color::RgbColor;
use crate::element::Element;
use crate::parsed::LineTagDefinition;

/// [`LineTagProperties`] for a line mode, as well as an [`Element`] if one is associated with the
/// current user-defined line mode.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LineTag<'a> {
    pub element: Option<&'a Element>,
    pub properties: &'a LineTagProperties,
}

impl Deref for LineTag<'_> {
    type Target = LineTagProperties;

    fn deref(&self) -> &Self::Target {
        self.properties
    }
}

/// Properties defined for a line tag.
///
/// See [MXP specification: Line Tags](https://www.zuggsoft.com/zmud/mxp.htm#Line%20Tags).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LineTagProperties {
    /// Use this tag.
    pub enable: bool,
    /// Override foreground color (from line tag)
    pub fore: Option<RgbColor>,
    /// Override background color (from line tag)
    pub back: Option<RgbColor>,
    /// Suppress output in main window (from line tag)
    pub gag: bool,
    /// Redirect output to another window (from line tag)
    pub window: Option<String>,
}

impl LineTagProperties {
    pub fn apply(&mut self, definition: LineTagDefinition) {
        if let Some(enable) = definition.enable {
            self.enable = enable;
        }
        if let Some(fore) = definition.fore {
            self.fore = Some(fore);
        }
        if let Some(back) = definition.back {
            self.back = Some(back);
        }
        if let Some(gag) = definition.gag {
            self.gag = gag;
        }
        if let Some(window) = definition.window {
            self.window = Some(window.to_owned());
        }
    }
}
