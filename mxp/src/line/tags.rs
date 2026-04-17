use super::mode::Mode;
use super::tag::{LineTag, LineTagProperties};
use crate::CaseFoldMap;
use crate::element::Element;
use crate::node::LineTagDefinition;
use crate::{Error, ErrorKind};

const OFFSET: usize = Mode::USER_DEFINED_MIN.0 as usize;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Line {
    pub element: String,
    pub properties: LineTagProperties,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LineTags {
    base: Vec<Line>,
}

impl Clone for LineTags {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.base.clone_from(&source.base);
    }
}

impl Default for LineTags {
    fn default() -> Self {
        Self::new()
    }
}

impl LineTags {
    pub const fn new() -> Self {
        Self { base: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.base.clear();
    }

    pub fn get<'a>(
        &'a self,
        mode: usize,
        elements: &'a CaseFoldMap<Element>,
    ) -> Option<LineTag<'a>> {
        let i = mode.checked_sub(OFFSET)?;
        let line = self.base.get(i)?;
        Some(LineTag {
            element: elements.get(&line.element),
            properties: &line.properties,
        })
    }

    pub fn set(&mut self, mode: usize, element: String) {
        if mode > Mode::USER_DEFINED_MAX {
            return;
        }
        let Some(i) = mode.checked_sub(OFFSET) else {
            return;
        };
        if self.base.len() <= i {
            self.base.resize_with(i + 1, Default::default);
        }
        let el = &mut self.base[i];
        el.element = element;
        el.properties.enable = true;
    }

    pub fn update(&mut self, update: LineTagDefinition) -> crate::Result<()> {
        fn create_error(update: &LineTagDefinition) -> Error {
            Error::new(update.index.0.to_string(), ErrorKind::IllegalLineTag)
        }

        if update.index > Mode::USER_DEFINED_MAX {
            return Err(create_error(&update));
        }
        let Some(i) = usize::from(update.index.0).checked_sub(OFFSET) else {
            return Err(create_error(&update));
        };
        if self.base.len() <= i {
            self.base.resize_with(i + 1, Default::default);
        }
        let tag = &mut self.base[i];
        tag.properties.apply(update);
        Ok(())
    }
}
