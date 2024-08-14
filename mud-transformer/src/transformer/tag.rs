/// Outstanding (unclosed) tags.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag {
    /// Name of tag we opened
    pub name: String,
    /// Was it secure mode at the time?
    pub secure: bool,
    /// Protected from reset?
    pub no_reset: bool,
    /// Index in a style's span list.
    pub span_index: usize,
}

impl Tag {
    pub fn new(
        component: mxp::ElementComponent,
        secure: bool,
        span_index: usize,
    ) -> mxp::Result<Self> {
        let name = component.name().to_owned();
        let flags = component.flags();
        if !flags.contains(mxp::TagFlag::Open) && !secure {
            return Err(mxp::Error::new(name, mxp::ErrorKind::ElementWhenNotSecure));
        }
        Ok(Self {
            name,
            secure,
            no_reset: flags.contains(mxp::TagFlag::NoReset),
            span_index,
        })
    }

    pub fn parse_closing_tag(tag_body: &str) -> mxp::Result<&str> {
        let mut words = mxp::Words::new(tag_body);
        let name = words.validate_next_or(mxp::ErrorKind::InvalidElementName)?;

        if words.next().is_some() {
            return Err(mxp::Error::new(
                tag_body,
                mxp::ErrorKind::ArgumentsToClosingTag,
            ));
        }

        Ok(name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TagList {
    inner: Vec<Tag>,
}

impl Default for TagList {
    fn default() -> Self {
        Self::new()
    }
}

impl TagList {
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn truncate(&mut self, pos: usize) -> Option<usize> {
        let span_index = self.inner.get(pos)?.span_index;
        self.inner.truncate(pos);
        Some(span_index)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn push(&mut self, tag: Tag) {
        self.inner.push(tag);
    }

    pub fn last_resettable_index(&self) -> usize {
        match self.inner.iter().rposition(|x| x.no_reset) {
            None => 0,
            Some(i) => i + 1,
        }
    }

    pub fn last_unsecure_index(&self) -> usize {
        match self.inner.iter().rposition(|x| x.secure) {
            None => 0,
            Some(i) => i + 1,
        }
    }

    pub fn find_last(&self, secure: bool, name: &str) -> mxp::Result<(usize, &Tag)> {
        for (i, tag) in self.inner.iter().enumerate().rev() {
            if tag.name.eq_ignore_ascii_case(name) {
                if !secure && tag.secure {
                    return Err(mxp::Error::new(name, mxp::ErrorKind::TagOpenedInSecureMode));
                }
                return Ok((i, tag));
            }
            if !secure && tag.secure {
                return Err(mxp::Error::new(
                    tag.name.clone(),
                    mxp::ErrorKind::OpenTagBlockedBySecureTag,
                ));
            }
        }
        Err(mxp::Error::new(name, mxp::ErrorKind::OpenTagNotThere))
    }
}
