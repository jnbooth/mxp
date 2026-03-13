use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

use crate::collections::CaseFoldMap;

#[derive(Clone, Debug)]
pub struct ArgumentMatcher<'a, I>
where
    I: Iterator<Item = &'a Cow<'a, str>>,
{
    inner: I,
    named: &'a CaseFoldMap<'a, Cow<'a, str>>,
}

impl<'a, I> ArgumentMatcher<'a, I>
where
    I: Iterator<Item = &'a Cow<'a, str>>,
{
    pub fn new<P>(positional: P, named: &'a CaseFoldMap<'a, Cow<'a, str>>) -> Self
    where
        P: IntoIterator<IntoIter = I>,
    {
        Self {
            inner: positional.into_iter(),
            named,
        }
    }

    pub fn into_inner(self) -> I {
        self.inner
    }

    pub fn map<F, U>(self, f: F) -> ArgumentMatcher<'a, U>
    where
        F: FnOnce(I) -> U,
        U: Iterator<Item = &'a Cow<'a, str>>,
    {
        ArgumentMatcher {
            inner: f(self.inner),
            named: self.named,
        }
    }
}

impl<'a, I> ArgumentMatcher<'a, I>
where
    I: Iterator<Item = &'a Cow<'a, str>>,
{
    pub fn next_or(&mut self, name: &str) -> Option<I::Item> {
        match self.named.get(name) {
            Some(arg) => Some(arg),
            None => self.inner.next(),
        }
    }
}

impl<'a, I> Deref for ArgumentMatcher<'a, I>
where
    I: Iterator<Item = &'a Cow<'a, str>>,
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, I> DerefMut for ArgumentMatcher<'a, I>
where
    I: Iterator<Item = &'a Cow<'a, str>>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
