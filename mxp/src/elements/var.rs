use std::borrow::Cow;
use std::str::FromStr;

use flagset::FlagSet;

use crate::keyword::EntityKeyword;
use crate::parse::{Decoder, Error, ExpectArg as _, Scan};

/// The `<VAR>` tag is just like the `<!ENTITY>` tag, except that the value of the variable is
/// placed between the `<VAR>` and `</VAR>` tags, and this value is displayed to the user.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Var<S = String> {
    /// Variable name.
    pub variable: S,
    /// Keywords.
    pub keywords: FlagSet<EntityKeyword>,
}

impl<S> Var<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, f: F) -> Var<T>
    where
        F: FnOnce(S) -> T,
    {
        Var {
            variable: f(self.variable),
            keywords: self.keywords,
        }
    }
}

impl_into_owned!(Var);

impl<S: AsRef<str>> Var<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Var<&str> {
        Var {
            variable: self.variable.as_ref(),
            keywords: self.keywords,
        }
    }
}

impl_partial_eq!(Var);

impl<'a, D> TryFrom<Scan<'a, D>> for Var<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        let variable = scanner.next()?.expect_some("variable")?;
        let keywords = scanner.into_keywords()?;
        Ok(Self { variable, keywords })
    }
}

impl FromStr for Var {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Var)
    }
}
