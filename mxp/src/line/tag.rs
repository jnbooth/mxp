use std::fmt;
use std::ops::Deref;

use crate::color::RgbColor;
use crate::element::Element;
use crate::node::LineTagDefinition;

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
    /// Redirect output to another window.
    pub window: Option<String>,
    /// Override foreground color.
    pub fore: Option<RgbColor>,
    /// Override background color.
    pub back: Option<RgbColor>,
    /// Suppress output in main window.
    pub gag: bool,
    /// Use this tag.
    pub enable: bool,
}

impl LineTagProperties {
    pub(crate) fn apply(&mut self, definition: LineTagDefinition) {
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

impl fmt::Display for LineTagProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::display::{DelimAfterFirst, Escape};

        let Self {
            window,
            fore,
            back,
            gag,
            enable,
        } = self;
        let delim = DelimAfterFirst::new(" ");
        if let Some(window) = window {
            write!(f, "{delim}WINDOWNAME={}", Escape(window))?;
        }
        if let Some(fore) = fore {
            write!(f, "{delim}FORE={fore}")?;
        }
        if let Some(back) = back {
            write!(f, "{delim}BACK={back}")?;
        }
        if *gag {
            write!(f, "{delim}GAG")?;
        }
        if !*enable {
            write!(f, "{delim}DISABLE")?;
        }
        Ok(())
    }
}
