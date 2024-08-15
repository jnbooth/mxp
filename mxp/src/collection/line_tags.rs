use super::element_map::ElementMap;
use crate::argument::scan::Decoder;
use crate::argument::Arguments;
use crate::color::RgbColor;
use crate::entity::{Element, Mode};
use crate::keyword::TagKeyword;
use crate::parser::{Error, ErrorKind, Words};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineTag {
    element: String,
    enabled: bool,
}

impl Default for LineTag {
    fn default() -> Self {
        Self::new()
    }
}

impl LineTag {
    pub const fn new() -> Self {
        Self {
            element: String::new(),
            enabled: false,
        }
    }
}

const OFFSET: usize = Mode::USER_DEFINED_MIN.0 as usize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineTags {
    inner: Vec<LineTag>,
}

impl Default for LineTags {
    fn default() -> Self {
        Self::new()
    }
}

impl LineTags {
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn get<'a>(&self, mode: usize, elements: &'a ElementMap) -> Option<&'a Element> {
        let i = mode.checked_sub(OFFSET)?;
        let tag = self.inner.get(i)?;
        if !tag.enabled {
            return None;
        }
        elements.get(&tag.element)
    }

    pub fn set(&mut self, mode: usize, element: String) {
        let Some(i) = mode.checked_sub(OFFSET) else {
            return;
        };
        if self.inner.len() <= i {
            self.inner.resize_with(i + 1, Default::default);
        }
        let el = &mut self.inner[i];
        el.element = element;
        el.enabled = true;
    }

    pub fn update<'a>(
        &mut self,
        update: LineTagUpdate,
        elements: &'a mut ElementMap,
    ) -> Option<&'a mut Element> {
        let i = (update.index as usize).checked_sub(OFFSET)?;
        let tag = self.inner.get_mut(i)?;
        if let Some(enable) = update.enable {
            tag.enabled = enable;
        }
        let element = elements.get_mut(&tag.element)?;
        if let Some(window) = update.window {
            element.window = Some(window.clone());
        }
        if let Some(gag) = update.gag {
            element.gag = gag;
        }
        if let Some(fore) = update.fore {
            element.fore = Some(fore);
        }
        if let Some(back) = update.back {
            element.back = Some(back);
        }
        Some(element)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineTagUpdate {
    index: u8,
    window: Option<String>,
    gag: Option<bool>,
    fore: Option<RgbColor>,
    back: Option<RgbColor>,
    enable: Option<bool>,
}

impl LineTagUpdate {
    pub fn parse<D: Decoder>(words: Words, decoder: D) -> crate::Result<Self> {
        let s = words.as_str();
        let args = Arguments::parse(words)?;
        let mut scanner = args.scan(decoder).with_keywords();

        let index_arg = scanner
            .next()?
            .ok_or(Error::new(s, ErrorKind::IncompleteArguments))?;
        let index_str = index_arg.as_ref();
        let index: u8 = index_str
            .parse()
            .map_err(|_| Error::new(index_str, ErrorKind::InvalidNumber))?;

        let window = scanner
            .next_or("windowname")?
            .map(|s| s.as_ref().to_owned());

        let fore = scanner
            .next_or("fore")?
            .and_then(|color| RgbColor::named(color.as_ref()));

        let back = scanner
            .next_or("back")?
            .and_then(|color| RgbColor::named(color.as_ref()));

        let keywords = scanner.into_keywords();
        let gag = if keywords.contains(TagKeyword::Gag) {
            Some(true)
        } else {
            None
        };
        let enable = if keywords.contains(TagKeyword::Disable) {
            Some(false)
        } else if keywords.contains(TagKeyword::Enable) {
            Some(true)
        } else {
            None
        };
        Ok(Self {
            index,
            window,
            gag,
            fore,
            back,
            enable,
        })
    }
}

/*
INDEX is the tag number (20-99) to change.

WINDOWNAME specifies the name of a window to redirect the text to.

GAG indicates that the text should be gagged from the main MUD window.

FORE is the text color.

BACK is the background color of the text.

ENABLE and DISABLE can be used to turn this tag on or off.

 */
