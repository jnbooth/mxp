use std::borrow::Cow;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct StyleVersion<S = String> {
    pub styleversion: S,
}

impl<S> StyleVersion<S> {
    pub fn map_text<T, F>(self, f: F) -> StyleVersion<T>
    where
        F: FnOnce(S) -> T,
    {
        StyleVersion {
            styleversion: f(self.styleversion),
        }
    }
}

impl_into_owned!(StyleVersion);
