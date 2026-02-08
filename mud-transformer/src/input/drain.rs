use std::io::{self, BufRead, IoSliceMut, Read, Write};

#[must_use = "if the output is unused, use self.clear() instead"]
pub struct Drain<'a> {
    pub(super) external_cursor: &'a mut usize,
    pub(super) cursor: usize,
    pub(super) buf: &'a mut Vec<u8>,
}

impl Drain<'_> {
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.cursor >= self.buf.len()
    }

    #[inline]
    fn slice(&self) -> &[u8] {
        &self.buf[self.cursor..]
    }

    pub fn write_to<W: Write>(&mut self, mut writer: W) -> io::Result<usize> {
        let n = writer.write(self.slice())?;
        self.cursor += n;
        Ok(n)
    }

    pub fn write_all_to<W: Write>(&mut self, mut writer: W) -> io::Result<()> {
        let mut slice = &self.buf[self.cursor..];
        while !slice.is_empty() {
            let n = writer.write(slice)?;
            slice = &slice[n..];
            self.cursor += n;
        }
        Ok(())
    }
}

impl bytes::Buf for Drain<'_> {
    fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.cursor)
    }

    fn chunk(&self) -> &[u8] {
        self.slice()
    }

    fn advance(&mut self, cnt: usize) {
        self.cursor += cnt;
    }
}

impl Drop for Drain<'_> {
    fn drop(&mut self) {
        *self.external_cursor = 0;
        if self.is_empty() {
            self.buf.clear();
        } else {
            self.buf.copy_within(self.cursor.., 0);
            self.buf.truncate(self.buf.len() - self.cursor);
        }
    }
}

impl Read for Drain<'_> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.slice().read(buf)?;
        self.cursor += n;
        Ok(n)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
        let n = self.slice().read_vectored(bufs)?;
        self.cursor += n;
        Ok(n)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.slice().read_exact(buf)?;
        self.cursor += buf.len();
        Ok(())
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let n = self.slice().read_to_end(buf)?;
        self.cursor += n;
        Ok(n)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let n = self.slice().read_to_string(buf)?;
        self.cursor += n;
        Ok(n)
    }
}

impl BufRead for Drain<'_> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(self.slice())
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.cursor += amt;
    }
}
