use std::collections::hash_map;
use std::iter::FusedIterator;
use std::{fmt, slice};

use uncased::Uncased;

/// This struct is created by [`Arguments::named`](crate::Arguments::named).
/// See its documentation for more.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Named<'a, S> {
    pub(super) iter: hash_map::Iter<'a, Uncased<'a>, S>,
}

impl<S> Clone for Named<'_, S> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<S: fmt::Debug> fmt::Debug for Named<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.clone()).finish()
    }
}

impl<'a, S> Iterator for Named<'a, S> {
    type Item = (&'a str, &'a S);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (k, v) = self.iter.next()?;
        Some((k.as_str(), v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }
}

impl<S> ExactSizeIterator for Named<'_, S> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<S> FusedIterator for Named<'_, S> {}

/// This struct is created by [`Arguments::positional`](crate::Arguments::positional).
/// See its documentation for more.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Positional<'a, S> {
    pub(super) iter: slice::Iter<'a, S>,
}

impl<S> Clone for Positional<'_, S> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<S: fmt::Debug> fmt::Debug for Positional<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.iter.fmt(f)
    }
}

impl<'a, S> Iterator for Positional<'a, S> {
    type Item = &'a S;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last()
    }
}

impl<S> DoubleEndedIterator for Positional<'_, S> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n)
    }
}

impl<S> FusedIterator for Positional<'_, S> {}
