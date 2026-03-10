use std::borrow::Cow;

use flagset::FlagSet;

use crate::argument::{Decoder, ExpectArg as _, Scan};
use crate::keyword::EntityKeyword;
use crate::parser::Error;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Var<S = String> {
    pub variable: S,
    pub keywords: FlagSet<EntityKeyword>,
}

impl<S> Var<S> {
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

impl<'a, D> TryFrom<Scan<'a, D>> for Var<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        Ok(Self {
            variable: scanner.next()?.expect_some("variable")?,
            keywords: scanner.into_keywords(),
        })
    }
}
