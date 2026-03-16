use std::borrow::Cow;
use std::slice;
use std::str::FromStr;

use flagset::{FlagSet, Flags};

use crate::arguments::{ArgumentMatcher, OwnedArgumentMatcher};
use crate::keyword::{KeywordFilter, KeywordFilterIter};
use crate::parse::{Decoder, KeywordScan, OwnedKeywordScan, OwnedScan, Scan};
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

impl<'a, D: Decoder> ArgumentScanner for OwnedScan<'a, D> {
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

impl<'a, D, K> ArgumentScanner for OwnedKeywordScan<'a, D, K>
where
    D: Decoder,
    K: KeywordFilter + Flags + FromStr,
{
    type Output = Cow<'a, str>;

    fn next(&mut self) -> Result<Option<Self::Output>> {
        self.next()
    }

    fn next_or(&mut self, name: &str) -> Result<Option<Self::Output>> {
        self.next_or(name)
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

impl<I, S> ArgumentScanner for OwnedArgumentMatcher<'_, I, S>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    type Output = S;

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

impl<D, K> ArgumentScannerWithKeywords for OwnedKeywordScan<'_, D, K>
where
    D: Decoder,
    K: KeywordFilter + Flags + FromStr,
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

impl<K, I, S> ArgumentScannerWithKeywords for OwnedArgumentMatcher<'_, KeywordFilterIter<K, I>, S>
where
    K: Flags + FromStr,
    I: Iterator<Item = S>,
    S: AsRef<str>,
    Error: From<K::Err>,
{
    type Keyword = K;

    fn into_keywords(self) -> Result<FlagSet<Self::Keyword>> {
        Ok(self.into_keywords()?)
    }
}

pub(crate) trait IntoArgumentScannerWithKeywords<K, O> {
    type WithKeywords: ArgumentScannerWithKeywords<Keyword = K, Output = O>;

    fn with_keywords(self) -> Self::WithKeywords;
}

impl<A, K> IntoArgumentScannerWithKeywords<K, A::Output> for A
where
    A: ArgumentScannerWithKeywords<Keyword = K>,
{
    type WithKeywords = Self;

    fn with_keywords(self) -> Self::WithKeywords {
        self
    }
}

impl<'a, D, K, S> IntoArgumentScannerWithKeywords<K, Cow<'a, str>> for Scan<'a, D, S>
where
    D: Decoder,
    K: Flags + FromStr + KeywordFilter,
    S: AsRef<str>,
    Error: From<K::Err>,
{
    type WithKeywords = KeywordScan<'a, D, K, S>;

    fn with_keywords(self) -> Self::WithKeywords {
        self.with_keywords()
    }
}

impl<'a, D, K> IntoArgumentScannerWithKeywords<K, Cow<'a, str>> for OwnedScan<'a, D>
where
    D: Decoder,
    K: Flags + FromStr + KeywordFilter,
    Error: From<K::Err>,
{
    type WithKeywords = OwnedKeywordScan<'a, D, K>;

    fn with_keywords(self) -> Self::WithKeywords {
        self.with_keywords()
    }
}

impl<'a, K, S> IntoArgumentScannerWithKeywords<K, &'a S>
    for ArgumentMatcher<'a, slice::Iter<'a, S>, S>
where
    K: Flags + FromStr + KeywordFilter,
    S: AsRef<str>,
    Error: From<K::Err>,
{
    type WithKeywords = ArgumentMatcher<'a, KeywordFilterIter<K, slice::Iter<'a, S>>, S>;

    fn with_keywords(self) -> Self::WithKeywords {
        self.with_keywords()
    }
}
