use std::borrow::Cow;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct StyleVersion<S = String> {
    pub styleversion: S,
}

impl StyleVersion<&str> {
    pub fn into_owned(self) -> StyleVersion<String> {
        StyleVersion {
            styleversion: self.styleversion.to_owned(),
        }
    }
}

impl StyleVersion<Cow<'_, str>> {
    pub fn into_owned(self) -> StyleVersion<String> {
        StyleVersion {
            styleversion: self.styleversion.into_owned(),
        }
    }
}
