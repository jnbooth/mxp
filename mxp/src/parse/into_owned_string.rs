use std::borrow::Cow;

/// Internal helper trait for functions such as [`Action::into_owned`](crate::Action::into_owned).
pub trait IntoOwnedString {
    fn into_owned_string(self) -> String;
}

impl IntoOwnedString for &str {
    fn into_owned_string(self) -> String {
        self.to_owned()
    }
}

impl IntoOwnedString for Cow<'_, str> {
    fn into_owned_string(self) -> String {
        self.into_owned()
    }
}
