use std::borrow::Cow;
use std::str::FromStr;

use flagset::FlagSet;

use crate::arguments::ExpectArg as _;
use crate::keyword::EntityKeyword;
use crate::parse::{Decoder, Scan};
use crate::parsed::EntityDefinition;

/// The `<VAR>` tag is just like the `<!ENTITY>` tag, except that the value of the variable is
/// placed between the `<VAR>` and `</VAR>` tags, and this value is displayed to the user.
///
/// # Examples
///
/// ```
/// use mxp::entity::EntityKeyword;
///
/// assert_eq!(
///     "<VAR Hp DESC=Health PUBLISH>".parse::<mxp::Var>(),
///     Ok(mxp::Var {
///         name: "Hp".into(),
///         desc: Some("Health".into()),
///         keywords: EntityKeyword::Publish.into(),
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Var<S = String> {
    /// Variable name.
    pub name: S,
    /// Variable description.
    pub desc: Option<S>,
    /// Keywords.
    pub keywords: FlagSet<EntityKeyword>,
}

impl<S> Var<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, mut f: F) -> Var<T>
    where
        F: FnMut(S) -> T,
    {
        Var {
            name: f(self.name),
            desc: self.desc.map(f),
            keywords: self.keywords,
        }
    }
}

impl_into_owned!(Var);

impl<S: AsRef<str>> Var<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Var<&str> {
        Var {
            name: self.name.as_ref(),
            desc: self.desc.as_ref().map(AsRef::as_ref),
            keywords: self.keywords,
        }
    }

    /// Creates an `EntityDefinition` from properties and a value.
    pub fn with_value<'a>(&'a self, value: &'a str) -> EntityDefinition<'a> {
        EntityDefinition {
            name: self.name.as_ref(),
            desc: self.desc.as_ref().map(AsRef::as_ref),
            keywords: self.keywords,
            value,
        }
    }
}

impl_partial_eq!(Var);

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Var<Cow<'a, str>> {
    type Error = crate::Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        let variable = scanner.next()?.expect_some("Variable")?;
        let desc = scanner.next_or("desc")?;
        let keywords = scanner.into_keywords()?;
        Ok(Self {
            name: variable,
            desc,
            keywords,
        })
    }
}

impl<S> From<S> for Var<S> {
    fn from(name: S) -> Self {
        Self {
            name,
            desc: None,
            keywords: FlagSet::empty(),
        }
    }
}

impl FromStr for Var {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Var)
    }
}
