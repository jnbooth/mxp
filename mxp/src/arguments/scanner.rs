use std::borrow::Cow;
use std::str::FromStr;

use flagset::{FlagSet, Flags};

use crate::{Error, ErrorKind};

pub(crate) trait ArgumentScanner<'a>: Sized {
    type Output: AsRef<str>;

    fn decode(&self, raw: Self::Output) -> crate::Result<Cow<'a, str>>;
    fn get_named(&mut self, name: &str) -> Option<Self::Output>;
    fn get_next(&mut self) -> Option<Self::Output>;
    fn get_next_or(&mut self, name: &str) -> Option<Self::Output> {
        self.get_named(name).or_else(|| self.get_next())
    }
    fn decode_next(&mut self) -> crate::Result<Option<Cow<'a, str>>> {
        match self.get_next() {
            Some(next) => Ok(Some(self.decode(next)?)),
            None => Ok(None),
        }
    }
    fn decode_next_or(&mut self, name: &str) -> crate::Result<Option<Cow<'a, str>>> {
        match self.get_next_or(name) {
            Some(next) => Ok(Some(self.decode(next)?)),
            None => Ok(None),
        }
    }
    fn expect_end(mut self) -> crate::Result<()> {
        match self.get_next() {
            Some(next) => Err(Error::new(next.as_ref(), ErrorKind::UnexpectedArgument)),
            None => Ok(()),
        }
    }
    fn with_keywords<K: Flags>(self) -> KeywordArgumentScanner<Self, K> {
        KeywordArgumentScanner {
            inner: self,
            keywords: FlagSet::empty(),
        }
    }
}

pub(crate) struct KeywordArgumentScanner<A, K: Flags> {
    inner: A,
    keywords: FlagSet<K>,
}

impl<'a, A, K> KeywordArgumentScanner<A, K>
where
    A: ArgumentScanner<'a>,
    K: Flags + FromStr,
{
    pub fn into_keywords(mut self) -> crate::Result<FlagSet<K>> {
        while let Some(next) = self.inner.decode_next()? {
            self.keywords |=
                K::from_str(&next).map_err(|_| Error::new(next, ErrorKind::UnexpectedArgument))?;
        }
        Ok(self.keywords)
    }
}

impl<'a, A, K> ArgumentScanner<'a> for KeywordArgumentScanner<A, K>
where
    A: ArgumentScanner<'a>,
    K: Flags + FromStr,
{
    type Output = A::Output;

    fn decode(&self, output: Self::Output) -> crate::Result<Cow<'a, str>> {
        self.inner.decode(output)
    }

    fn get_named(&mut self, name: &str) -> Option<Self::Output> {
        self.inner.get_named(name)
    }

    fn get_next(&mut self) -> Option<Self::Output> {
        while let Some(next) = self.inner.get_next() {
            match next.as_ref().parse::<K>() {
                Ok(keyword) => self.keywords |= keyword,
                Err(_) => return Some(next),
            }
        }
        None
    }
}
