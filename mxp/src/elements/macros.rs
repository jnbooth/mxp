macro_rules! impl_into_owned {
    ($t:ident) => {
        impl $t<&str> {
            pub fn into_owned(self) -> $t<String> {
                self.map_text(ToOwned::to_owned)
            }
        }

        impl $t<std::borrow::Cow<'_, str>> {
            pub fn into_owned(self) -> $t<String> {
                self.map_text(Cow::into_owned)
            }
        }

        impl<D: crate::parse::Decoder> TryFrom<crate::parse::Scan<'_, D>> for $t<String> {
            type Error = crate::Error;

            fn try_from(scanner: crate::parse::Scan<'_, D>) -> crate::Result<Self> {
                Ok($t::<Cow<'_, str>>::try_from(scanner)?.into_owned())
            }
        }
    };
}

macro_rules! impl_partial_eq {
    ($t:ident) => {
        impl PartialEq<$t<&str>> for $t<std::borrow::Cow<'_, str>> {
            fn eq(&self, other: &$t<&str>) -> bool {
                self.borrow_text() == *other
            }
        }

        impl PartialEq<$t<std::borrow::Cow<'_, str>>> for $t<&str> {
            fn eq(&self, other: &$t<std::borrow::Cow<'_, str>>) -> bool {
                *other == *self
            }
        }

        impl PartialEq<$t<&str>> for $t<String> {
            fn eq(&self, other: &$t<&str>) -> bool {
                self.borrow_text() == *other
            }
        }

        impl PartialEq<$t<String>> for $t<&str> {
            fn eq(&self, other: &$t<String>) -> bool {
                *other == *self
            }
        }

        impl PartialEq<$t<String>> for $t<std::borrow::Cow<'_, str>> {
            fn eq(&self, other: &$t<String>) -> bool {
                self.borrow_text() == other.borrow_text()
            }
        }

        impl PartialEq<$t<std::borrow::Cow<'_, str>>> for $t<String> {
            fn eq(&self, other: &$t<std::borrow::Cow<'_, str>>) -> bool {
                *other == *self
            }
        }
    };
}
