use std::borrow::Cow;

use crate::argument::{Decoder, ExpectArg as _, Scan};
use crate::parser::Error;

/// Defines a graphics format and provides a client plugin module that converts the MUD-specific
/// format to a standard GIF or BMP format.
///
/// See [MXP specification: `<FILTER>`](https://www.zuggsoft.com/zmud/mxp.htm#File%20Filters).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter<S = String> {
    pub src: S,
    pub dest: S,
    pub name: S,
}

impl<S> Filter<S> {
    pub fn map_text<T, F>(self, mut f: F) -> Filter<T>
    where
        F: FnMut(S) -> T,
    {
        Filter {
            src: f(self.src),
            dest: f(self.dest),
            name: f(self.name),
        }
    }
}

impl_into_owned!(Filter);

impl<'a, D> TryFrom<Scan<'a, D>> for Filter<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            src: scanner.next_or("src")?.expect_some("src")?,
            dest: scanner.next_or("dest")?.expect_some("dest")?,
            name: scanner.next_or("name")?.expect_some("name")?,
        })
    }
}
