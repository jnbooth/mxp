use std::borrow::Cow;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use flagset::{FlagSet, Flags};

use crate::collections::CaseFoldMap;
use crate::keyword::{KeywordFilter, KeywordFilterIter};

#[derive(Clone, Debug)]
pub struct ArgumentMatcher<'a, I, S = Cow<'a, str>>
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

    pub fn map<F, U>(self, f: F) -> ArgumentMatcher<'a, U, S>
    where
        F: FnOnce(I) -> U,
        U: Iterator<Item = &'a S>,
    {
        ArgumentMatcher {
            inner: f(self.inner),
            named: self.named,
        }
    }

    pub fn next_or(&mut self, name: &str) -> Option<I::Item> {
        match self.named.get(name) {
            Some(arg) => Some(arg),
            None => self.inner.next(),
        }
    }

    pub fn with_keywords<K>(self) -> ArgumentMatcher<'a, KeywordFilterIter<K, I>, S>
    where
        K: Flags + FromStr + KeywordFilter,
        S: AsRef<str>,
    {
        self.map(KeywordFilterIter::new)
    }
}

impl<'a, K, I, S> ArgumentMatcher<'a, KeywordFilterIter<K, I>, S>
where
    K: Flags + FromStr,
    I: Iterator<Item = &'a S>,
    S: AsRef<str>,
{
    pub fn into_keywords(self) -> Result<FlagSet<K>, K::Err> {
        self.inner.into_keywords()
    }
}

impl<'a, I, S> Deref for ArgumentMatcher<'a, I, S>
where
    I: Iterator<Item = &'a S>,
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, I, S> DerefMut for ArgumentMatcher<'a, I, S>
where
    I: Iterator<Item = &'a S>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
