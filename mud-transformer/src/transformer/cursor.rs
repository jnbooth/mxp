use std::io::{self, BufRead, IoSliceMut, Read};
use std::iter::FusedIterator;

#[derive(Clone, Debug, Default)]
pub(crate) struct ReceiveCursor<'a> {
    inner: &'a [u8],
}

impl<'a> ReceiveCursor<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { inner: bytes }
    }

    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub const fn as_slice(&self) -> &'a [u8] {
        self.inner
    }
}

impl Read for ReceiveCursor<'_> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
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

impl BufRead for ReceiveCursor<'_> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
    }
}

impl Iterator for ReceiveCursor<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let [first, tail @ ..] = self.inner {
            self.inner = tail;
            Some(*first)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.len();
        (exact, Some(exact))
    }
}

impl ExactSizeIterator for ReceiveCursor<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl FusedIterator for ReceiveCursor<'_> {}
