use std::borrow::Cow;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct StyleVersion<S = String> {
    pub styleversion: S,
}

impl<S> StyleVersion<S> {
    /// Applies a type transformation to the text, returning a new struct.
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

impl<S: AsRef<str>> StyleVersion<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> StyleVersion<&str> {
        StyleVersion {
            styleversion: self.styleversion.as_ref(),
        }
    }
}

impl_partial_eq!(StyleVersion);
