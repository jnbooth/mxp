use std::str::FromStr;

use flagset::{FlagSet, Flags};

use crate::{Error, ErrorKind};

pub(crate) trait ArgumentScanner<'a>: Sized {
    type Raw: AsRef<str>;
    type Decoded: AsRef<str>;

    fn decode(&self, raw: Self::Raw) -> crate::Result<Self::Decoded>;
    fn raw_get_named(&mut self, name: &str) -> Option<Self::Raw>;
    fn raw_get_next(&mut self) -> Option<Self::Raw>;
    fn raw_get_next_or(&mut self, name: &str) -> Option<Self::Raw> {
        self.raw_get_named(name).or_else(|| self.raw_get_next())
    }
    fn get_named(&mut self, name: &str) -> crate::Result<Option<Self::Decoded>> {
        match self.raw_get_named(name) {
            Some(named) => Ok(Some(self.decode(named)?)),
            None => Ok(None),
        }
    }
    fn get_next(&mut self) -> crate::Result<Option<Self::Decoded>> {
        match self.raw_get_next() {
            Some(next) => Ok(Some(self.decode(next)?)),
            None => Ok(None),
        }
    }
    fn get_next_or(&mut self, name: &str) -> crate::Result<Option<Self::Decoded>> {
        match self.raw_get_next_or(name) {
            Some(next) => Ok(Some(self.decode(next)?)),
            None => Ok(None),
        }
    }
    fn expect_end(mut self) -> crate::Result<()> {
        match self.raw_get_next() {
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

    fn parse<T>(self) -> crate::Result<T>
    where
        T: FromArgs<'a, Self::Decoded>,
    {
        T::from_args(self)
    }
}

pub(crate) trait FromArgs<'a, S>: Sized {
    fn from_args<A: ArgumentScanner<'a, Decoded = S>>(scanner: A) -> crate::Result<Self>;
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
        while let Some(next) = self.inner.get_next()? {
            let next = next.as_ref();
            self.keywords |=
                K::from_str(next).map_err(|_| Error::new(next, ErrorKind::UnexpectedArgument))?;
        }
        Ok(self.keywords)
    }
}

impl<'a, A, K> ArgumentScanner<'a> for KeywordArgumentScanner<A, K>
where
    A: ArgumentScanner<'a>,
    K: Flags + FromStr,
{
    type Raw = A::Raw;
    type Decoded = A::Decoded;

    fn decode(&self, output: Self::Raw) -> crate::Result<Self::Decoded> {
        self.inner.decode(output)
    }

    fn raw_get_named(&mut self, name: &str) -> Option<Self::Raw> {
        self.inner.raw_get_named(name)
    }

    fn raw_get_next(&mut self) -> Option<Self::Raw> {
        while let Some(next) = self.inner.raw_get_next() {
            match next.as_ref().parse::<K>() {
                Ok(keyword) => self.keywords |= keyword,
                Err(_) => return Some(next),
            }
        }
        None
    }
}
