macro_rules! impl_into_owned {
    ($t:ident) => {
        impl<S: crate::parse::IntoOwnedString> $t<S> {
            pub fn into_owned(self) -> $t<String> {
                self.map_text(crate::parse::IntoOwnedString::into_owned_string)
            }
        }

        impl<D: crate::parse::Decoder> TryFrom<crate::parse::Scan<'_, D, Cow<'_, str>>>
            for $t<String>
        {
            type Error = crate::Error;

            fn try_from(scanner: crate::parse::Scan<'_, D, Cow<'_, str>>) -> crate::Result<Self> {
                Ok($t::<Cow<'_, str>>::try_from(scanner)?.into_owned())
            }
        }
    };
}

macro_rules! impl_partial_eq {
    ($t:ident) => {
        impl PartialEq<$t<&str>> for $t<std::borrow::Cow<'_, str>> {
            #[inline]
            fn eq(&self, other: &$t<&str>) -> bool {
                self.borrow_text() == *other
            }
        }

        impl PartialEq<$t<std::borrow::Cow<'_, str>>> for $t<&str> {
            #[inline]
            fn eq(&self, other: &$t<std::borrow::Cow<'_, str>>) -> bool {
                *other == *self
            }
        }

        impl PartialEq<$t<&str>> for $t<String> {
            #[inline]
            fn eq(&self, other: &$t<&str>) -> bool {
                self.borrow_text() == *other
            }
        }

        impl PartialEq<$t<String>> for $t<&str> {
            #[inline]
            fn eq(&self, other: &$t<String>) -> bool {
                *other == *self
            }
        }

        impl PartialEq<$t<String>> for $t<std::borrow::Cow<'_, str>> {
            #[inline]
            fn eq(&self, other: &$t<String>) -> bool {
                self.borrow_text() == other.borrow_text()
            }
        }

        impl PartialEq<$t<std::borrow::Cow<'_, str>>> for $t<String> {
            #[inline]
            fn eq(&self, other: &$t<std::borrow::Cow<'_, str>>) -> bool {
                *other == *self
            }
        }
    };
}

macro_rules! impl_from_str {
    ($t:ident) => {
        impl<'a, D: Decoder, S: AsRef<str>> TryFrom<crate::parse::Scan<'a, D, S>>
            for $t<std::borrow::Cow<'a, str>>
        {
            type Error = crate::Error;

            #[inline]
            fn try_from(scanner: crate::parse::Scan<'a, D, S>) -> crate::Result<Self> {
                Self::scan(scanner)
            }
        }

        impl<'a, D: Decoder> TryFrom<crate::parse::OwnedScan<'a, D>>
            for $t<std::borrow::Cow<'a, str>>
        {
            type Error = crate::Error;

            #[inline]
            fn try_from(scanner: crate::parse::OwnedScan<'a, D>) -> Result<Self, Self::Error> {
                Self::scan(scanner)
            }
        }

        impl<'a> TryFrom<&'a str> for $t<std::borrow::Cow<'a, str>> {
            type Error = crate::parse::FromStrError;

            #[inline]
            fn try_from(s: &'a str) -> Result<Self, Self::Error> {
                crate::parse::parse_element(s)
            }
        }

        impl std::str::FromStr for $t {
            type Err = crate::parse::FromStrError;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(crate::parse::parse_element::<$t<std::borrow::Cow<str>>>(s)?.into_owned())
            }
        }
    };
}
