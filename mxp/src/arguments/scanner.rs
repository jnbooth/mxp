use std::borrow::Cow;
use std::str::FromStr;

use flagset::{FlagSet, Flags};

use crate::arguments::ArgumentMatcher;
use crate::keyword::{KeywordFilter, KeywordFilterIter};
use crate::parse::{Decoder, KeywordScan, Scan};
use crate::{Error, ErrorKind, Result};

pub(crate) trait ArgumentScanner: Sized {
    type Output: AsRef<str>;

    fn next(&mut self) -> Result<Option<Self::Output>>;
    fn next_or(&mut self, name: &str) -> Result<Option<Self::Output>>;
    fn expect_end(mut self) -> Result<()> {
        if let Some(next) = self.next()? {
            return Err(Error::new(next.as_ref(), ErrorKind::UnexpectedArgument));
        }
        Ok(())
    }
}

impl<'a, D, K, S> ArgumentScanner for KeywordScan<'a, D, K, S>
where
    D: Decoder,
    K: Flags + FromStr + KeywordFilter,
    S: AsRef<str>,
    Error: From<K::Err>,
{
    type Output = Cow<'a, str>;

    fn next(&mut self) -> Result<Option<Self::Output>> {
        self.next()
    }

    fn next_or(&mut self, name: &str) -> Result<Option<Self::Output>> {
        self.next_or(name)
    }

    fn expect_end(self) -> Result<()> {
        self.into_keywords()?;
        Ok(())
    }
}

impl<'a, D, S> ArgumentScanner for Scan<'a, D, S>
where
    D: Decoder,
    S: AsRef<str>,
{
    type Output = Cow<'a, str>;

    fn next(&mut self) -> Result<Option<Self::Output>> {
        self.next()
    }

    fn next_or(&mut self, name: &str) -> Result<Option<Self::Output>> {
        self.next_or(name)
    }

    fn expect_end(self) -> Result<()> {
        self.expect_end()
    }
}

impl<'a, I, S> ArgumentScanner for ArgumentMatcher<'a, I, S>
where
    I: Iterator<Item = &'a S>,
    S: AsRef<str>,
{
    type Output = &'a S;

    fn next(&mut self) -> Result<Option<Self::Output>> {
        Ok(self.next())
    }

    fn next_or(&mut self, name: &str) -> Result<Option<Self::Output>> {
        Ok(self.next_or(name))
    }
}

pub(crate) trait ArgumentScannerWithKeywords: ArgumentScanner {
    type Keyword: Flags;

    fn into_keywords(self) -> Result<FlagSet<Self::Keyword>>;
}

impl<D, K, S> ArgumentScannerWithKeywords for KeywordScan<'_, D, K, S>
where
    D: Decoder,
    S: AsRef<str>,
    K: Flags + FromStr + KeywordFilter,
    Error: From<K::Err>,
{
    type Keyword = K;

    fn into_keywords(self) -> Result<FlagSet<Self::Keyword>> {
        Ok(self.into_keywords()?)
    }
}

impl<'a, K, I, S> ArgumentScannerWithKeywords for ArgumentMatcher<'a, KeywordFilterIter<K, I>, S>
where
    K: Flags + FromStr,
    I: Iterator<Item = &'a S>,
    S: AsRef<str>,
    Error: From<K::Err>,
{
    type Keyword = K;

    fn into_keywords(self) -> Result<FlagSet<Self::Keyword>> {
        Ok(self.into_keywords()?)
    }
}
