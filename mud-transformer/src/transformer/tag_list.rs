/// Outstanding (unclosed) tags.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Tag {
    /// Name of tag we opened
    pub name: String,
    /// Was it secure mode at the time?
    pub secure: bool,
    /// Index in a style's span list.
    pub span_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TagList {
    base: Vec<Tag>,
}

impl Default for TagList {
    fn default() -> Self {
        Self::new()
    }
}

impl TagList {
    pub const fn new() -> Self {
        Self { base: Vec::new() }
    }

    pub fn truncate(&mut self, pos: usize) -> Option<usize> {
        let span_index = self.base.get(pos)?.span_index;
        self.base.truncate(pos);
        Some(span_index)
    }

    pub fn clear(&mut self) {
        self.base.clear();
    }

    pub fn open(&mut self, component: mxp::Component, secure: bool, span_index: usize) {
        self.base.push(Tag {
            name: component.name().to_owned(),
            secure,
            span_index,
        });
    }

    pub fn last_open_index(&self) -> usize {
        match self.base.iter().rposition(|x| x.secure) {
            None => 0,
            Some(i) => i + 1,
        }
    }

    pub fn find_last(&self, secure: bool, name: &str) -> mxp::Result<usize> {
        for (i, tag) in self.base.iter().enumerate().rev() {
            if tag.name.eq_ignore_ascii_case(name) {
                if !secure && tag.secure {
                    return Err(mxp::Error::new(name, mxp::ErrorKind::TagOpenedInSecureMode));
                }
                return Ok(i);
            }
            if !secure && tag.secure {
                return Err(mxp::Error::new(
                    tag.name.clone(),
                    mxp::ErrorKind::OpenTagBlockedBySecureTag,
                ));
            }
        }
        Err(mxp::Error::new(name, mxp::ErrorKind::UnmatchedCloseTag))
    }
}
