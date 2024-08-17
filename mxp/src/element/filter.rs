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

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for Filter<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            src: scanner.next_or("src")?.expect_arg("src")?,
            dest: scanner.next_or("dest")?.expect_arg("dest")?,
            name: scanner.next_or("name")?.expect_arg("name")?,
        })
    }
}
