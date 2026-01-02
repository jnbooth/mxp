use std::borrow::Cow;

use crate::argument::{Decoder, ExpectArg, Scan};
use crate::parser::Error;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Filter<S = String> {
    pub src: S,
    pub dest: S,
    pub name: S,
}

impl Filter<&str> {
    pub fn into_owned(self) -> Filter {
        Filter {
            src: self.src.to_owned(),
            dest: self.dest.to_owned(),
            name: self.name.to_owned(),
        }
    }
}

impl Filter<Cow<'_, str>> {
    pub fn into_owned(self) -> Filter {
        Filter {
            src: self.src.into_owned(),
            dest: self.dest.into_owned(),
            name: self.name.into_owned(),
        }
    }
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for Filter<Cow<'a, str>>
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        Ok(Self {
            src: scanner.next_or("src")?.expect_some("src")?,
            dest: scanner.next_or("dest")?.expect_some("dest")?,
            name: scanner.next_or("name")?.expect_some("name")?,
        })
    }
}
