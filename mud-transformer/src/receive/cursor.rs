use std::io::{self, BufRead, IoSliceMut, Read};
use std::iter::FusedIterator;

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReceiveCursor<'a> {
    inner: &'a [u8],
}

impl<'a> ReceiveCursor<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { inner: bytes }
    }

    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<'a> Read for ReceiveCursor<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.inner.read_vectored(bufs)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_to_string(buf)
    }
}

impl<'a> BufRead for ReceiveCursor<'a> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
    }
}

impl<'a> Iterator for ReceiveCursor<'a> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let byte = *self.inner.first()?;
        self.inner = &self.inner[1..];
        Some(byte)
    }
}

impl<'a> FusedIterator for ReceiveCursor<'a> {}
