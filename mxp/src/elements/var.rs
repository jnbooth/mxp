use std::borrow::Cow;
use std::fmt;

use flagset::FlagSet;

use crate::arguments::{ArgumentScanner, ExpectArg as _};
use crate::keyword::EntityKeyword;
use crate::parse::Decoder;
use crate::parsed::ParsedEntityDefinition;

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
    pub fn with_value<'a>(&'a self, value: &'a str) -> ParsedEntityDefinition<'a> {
        ParsedEntityDefinition {
            name: self.name.as_ref(),
            desc: self.desc.as_ref().map(AsRef::as_ref),
            keywords: self.keywords,
            value,
        }
    }
}

impl_partial_eq!(Var);

impl<S: AsRef<str>> Var<S> {
    pub(crate) fn scan<A>(scanner: A) -> crate::Result<Self>
    where
        A: ArgumentScanner<Output = S>,
    {
        let mut scanner = scanner.with_keywords();
        let variable = scanner.decode_next()?.expect_some("Variable")?;
        let desc = scanner.decode_next_or("desc")?;
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

impl_from_str!(Var);

impl<S: AsRef<str>> fmt::Display for Var<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Var {
            name,
            desc,
            keywords,
        } = self.borrow_text().map_text(crate::display::Escape);
        write!(f, "<VAR {name}")?;
        if let Some(desc) = desc {
            write!(f, " {desc}")?;
        }
        for keyword in keywords {
            write!(f, " {keyword}")?;
        }
        f.write_str(">")
    }
}
