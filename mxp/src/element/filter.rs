use crate::argument::{Decoder, ExpectArg, Scan};
use crate::parser::Error;
use std::borrow::Cow;

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

impl<'a> Filter<Cow<'a, str>> {
    pub fn into_owned(self) -> Filter {
        Filter {
            src: self.src.into_owned(),
            dest: self.dest.into_owned(),
            name: self.name.into_owned(),
        }
    }
}

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for Filter<Cow<'a, str>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        Ok(Self {
            src: scanner.next_or("src")?.expect_some("src")?,
            dest: scanner.next_or("dest")?.expect_some("dest")?,
            name: scanner.next_or("name")?.expect_some("name")?,
        })
    }
}
