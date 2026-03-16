use std::str::FromStr;

use flagset::{FlagSet, Flags};

use crate::{Error, ErrorKind, KeywordFilter};

pub(crate) trait ArgumentScanner: Sized {
    type Output: AsRef<str>;
    type RawOutput: AsRef<str>;

    fn decode<F: KeywordFilter>(&self, raw: Self::RawOutput) -> crate::Result<Self::Output>;
    fn get_named(&mut self, name: &str) -> Option<Self::RawOutput>;
    fn get_next(&mut self) -> Option<Self::RawOutput>;
    fn get_next_or(&mut self, name: &str) -> Option<Self::RawOutput> {
        self.get_named(name).or_else(|| self.get_next())
    }
    fn decode_next(&mut self) -> crate::Result<Option<Self::Output>> {
        match self.get_next() {
            Some(next) => Ok(Some(self.decode::<()>(next)?)),
            None => Ok(None),
        }
    }
    fn decode_next_or(&mut self, name: &str) -> crate::Result<Option<Self::Output>> {
        match self.get_next_or(name) {
            Some(next) => Ok(Some(self.decode::<()>(next)?)),
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

impl<A, K> KeywordArgumentScanner<A, K>
where
    A: ArgumentScanner,
    A::Output: AsRef<str>,
    K: Flags + FromStr,
{
    pub fn into_keywords(mut self) -> crate::Result<FlagSet<K>> {
        while let Some(next) = self.inner.decode_next()? {
            let next = next.as_ref();
            self.keywords |=
                K::from_str(next).map_err(|_| Error::new(next, ErrorKind::UnexpectedArgument))?;
        }
        Ok(self.keywords)
    }
}

impl<A, K> ArgumentScanner for KeywordArgumentScanner<A, K>
where
    A: ArgumentScanner,
    A::Output: AsRef<str>,
    K: Flags + FromStr + KeywordFilter,
{
    type Output = A::Output;
    type RawOutput = A::RawOutput;

    fn decode<F: KeywordFilter>(&self, output: Self::RawOutput) -> crate::Result<Self::Output> {
        self.inner.decode::<K>(output)
    }

    fn get_named(&mut self, name: &str) -> Option<Self::RawOutput> {
        self.inner.get_named(name)
    }

    fn get_next(&mut self) -> Option<Self::RawOutput> {
        while let Some(next) = self.inner.get_next() {
            match next.as_ref().parse::<K>() {
                Ok(keyword) => self.keywords |= keyword,
                Err(_) => return Some(next),
            }
        }
        None
    }
}
