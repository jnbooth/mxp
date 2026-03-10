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

impl Var<&str> {
    pub fn into_owned(self) -> Var<String> {
        Var {
            variable: self.variable.to_owned(),
            keywords: self.keywords,
        }
    }
}

impl Var<Cow<'_, str>> {
    pub fn into_owned(self) -> Var<String> {
        Var {
            variable: self.variable.into_owned(),
            keywords: self.keywords,
        }
    }
}

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
