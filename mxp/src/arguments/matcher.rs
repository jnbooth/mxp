use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

use crate::CaseFoldMap;
use crate::arguments::ArgumentScanner;
use crate::keyword::KeywordFilter;

#[derive(Clone, Debug)]
pub(crate) struct ArgumentMatcher<'a, I, S = Cow<'a, str>>
where
    I: Iterator<Item = &'a S>,
{
    inner: I,
    named: &'a CaseFoldMap<'a, S>,
}

impl<'a, I, S> ArgumentMatcher<'a, I, S>
where
    I: Iterator<Item = &'a S>,
{
    pub fn new<P>(positional: P, named: &'a CaseFoldMap<'a, S>) -> Self
    where
        P: IntoIterator<IntoIter = I>,
    {
        Self {
            inner: positional.into_iter(),
            named,
        }
    }

    pub fn next(&mut self) -> Option<I::Item> {
        self.inner.next()
    }
}

impl<'a, I, S> ArgumentScanner for ArgumentMatcher<'a, I, S>
where
    I: Iterator<Item = &'a S>,
    S: AsRef<str>,
{
    type Output = &'a S;
    type RawOutput = Self::Output;

    fn decode<F: KeywordFilter>(&self, raw: Self::RawOutput) -> crate::Result<Self::Output> {
        Ok(raw)
    }

    fn get_next(&mut self) -> Option<Self::RawOutput> {
        self.inner.next()
    }
    fn get_next_or(&mut self, name: &str) -> Option<Self::RawOutput> {
        match self.named.get(name) {
            Some(arg) => Some(arg),
            None => self.inner.next(),
        }
    }
    fn decode_next(&mut self) -> crate::Result<Option<Self::Output>> {
        Ok(self.next())
    }
    fn decode_next_or(&mut self, name: &str) -> crate::Result<Option<Self::Output>> {
        Ok(self.get_next_or(name))
    }
}

impl<'a, I, S> Deref for ArgumentMatcher<'a, I, S>
where
    I: Iterator<Item = &'a S>,
{
    type Target = I;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, I, S> DerefMut for ArgumentMatcher<'a, I, S>
where
    I: Iterator<Item = &'a S>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Clone, Debug)]
pub(crate) struct OwnedArgumentMatcher<'a, I, S = Cow<'a, str>>
where
    I: Iterator<Item = S>,
{
    inner: I,
    named: CaseFoldMap<'a, S>,
}

impl<'a, I, S> OwnedArgumentMatcher<'a, I, S>
where
    I: Iterator<Item = S>,
{
    pub fn new<P>(positional: P, named: CaseFoldMap<'a, S>) -> Self
    where
        P: IntoIterator<IntoIter = I>,
    {
        Self {
            inner: positional.into_iter(),
            named,
        }
    }

    pub fn next(&mut self) -> Option<I::Item> {
        self.inner.next()
    }
}

impl<I, S> Deref for OwnedArgumentMatcher<'_, I, S>
where
    I: Iterator<Item = S>,
{
    type Target = I;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<I, S> DerefMut for OwnedArgumentMatcher<'_, I, S>
where
    I: Iterator<Item = S>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<I, S> ArgumentScanner for OwnedArgumentMatcher<'_, I, S>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    type Output = S;
    type RawOutput = Self::Output;

    fn decode<F: KeywordFilter>(&self, raw: Self::RawOutput) -> crate::Result<Self::Output> {
        Ok(raw)
    }

    fn get_next(&mut self) -> Option<Self::RawOutput> {
        self.inner.next()
    }
    fn get_next_or(&mut self, name: &str) -> Option<Self::RawOutput> {
        match self.named.remove(name) {
            Some(arg) => Some(arg),
            None => self.inner.next(),
        }
    }
    fn decode_next(&mut self) -> crate::Result<Option<Self::Output>> {
        Ok(self.next())
    }
    fn decode_next_or(&mut self, name: &str) -> crate::Result<Option<Self::Output>> {
        Ok(self.get_next_or(name))
    }
}
