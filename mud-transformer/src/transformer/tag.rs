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
    /// Special replacement sequence for clickable links that use the text they contain.
    pub anchor_template: Option<String>,
}

impl Tag {
    pub fn new(
        component: mxp::ElementComponent,
        secure: bool,
        span_index: usize,
    ) -> Result<Self, mxp::ParseError> {
        let name = component.name().to_owned();
        let flags = component.flags();
        if !flags.contains(mxp::TagFlag::Open) && !secure {
            return Err(mxp::ParseError::new(name, mxp::Error::ElementWhenNotSecure));
        }
        Ok(Self {
            name,
            secure,
            no_reset: flags.contains(mxp::TagFlag::NoReset),
            span_index,
            anchor_template: None,
        })
    }

    pub fn parse_closing_tag(tag_body: &str) -> Result<&str, mxp::ParseError> {
        let mut words = mxp::Words::new(tag_body);
        let name = words.validate_next_or(mxp::Error::InvalidElementName)?;

        if words.next().is_some() {
            return Err(mxp::ParseError::new(
                tag_body,
                mxp::Error::ArgumentsToClosingTag,
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

    pub fn set_anchor_template(&mut self, template: String) -> bool {
        match self.inner.last_mut() {
            Some(tag) => {
                tag.anchor_template = Some(template);
                true
            }
            None => false,
        }
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

    pub fn find_last(&self, secure: bool, name: &str) -> Result<(usize, &Tag), mxp::ParseError> {
        for (i, tag) in self.inner.iter().enumerate().rev() {
            if tag.name.eq_ignore_ascii_case(name) {
                if !secure && tag.secure {
                    return Err(mxp::ParseError::new(
                        name,
                        mxp::Error::TagOpenedInSecureMode,
                    ));
                } else {
                    return Ok((i, tag));
                }
            }
            if !secure && tag.secure {
                return Err(mxp::ParseError::new(
                    tag.name.clone(),
                    mxp::Error::OpenTagBlockedBySecureTag,
                ));
            }
        }
        Err(mxp::ParseError::new(name, mxp::Error::OpenTagNotThere))
    }
}
