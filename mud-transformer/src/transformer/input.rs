use std::io::{self, BufRead, IoSliceMut, Read, Write};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferedInput {
    buf: Vec<u8>,
    remember_latest: bool,
    latest: Option<Instant>,
    cursor: usize,
}

impl Default for BufferedInput {
    fn default() -> Self {
        Self::new(false)
    }
}

impl BufferedInput {
    pub const fn new(remember_latest: bool) -> Self {
        Self {
            buf: Vec::new(),
            remember_latest,
            latest: None,
            cursor: 0,
        }
    }

    pub fn afk(&self) -> Option<Duration> {
        self.latest.map(|latest| latest.elapsed())
    }

    pub fn set_remember(&mut self, remember_latest: bool) {
        self.remember_latest = remember_latest;
        if !remember_latest {
            self.latest = None;
        }
    }

    pub fn append(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub fn append_vectored(&mut self, messages: &[&str]) {
        for message in messages {
            self.buf.extend_from_slice(message.as_bytes());
        }
    }

    pub fn get_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buf
    }

    pub fn drain(&mut self) -> Drain {
        Drain {
            cursor: self.cursor,
            remember_latest: self.remember_latest,
            latest: &mut self.latest,
            external_cursor: &mut self.cursor,
            buf: &mut self.buf,
        }
    }
}

pub struct Drain<'a> {
    remember_latest: bool,
    latest: &'a mut Option<Instant>,
    external_cursor: &'a mut usize,
    cursor: usize,
    buf: &'a mut Vec<u8>,
}

impl<'a> Drain<'a> {
    #[inline]
    pub fn is_empty(&self) -> bool {
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
        writer.write_all(self.slice())?;
        self.cursor = self.buf.len();
        let mut slice = &self.buf[self.cursor..];
        while !slice.is_empty() {
            let n = writer.write(slice)?;
            slice = &slice[n..];
            self.cursor += n;
        }
        Ok(())
    }
}

impl<'a> bytes::Buf for Drain<'a> {
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

impl<'a> Drop for Drain<'a> {
    fn drop(&mut self) {
        if self.remember_latest {
            *self.latest = Some(Instant::now());
        }
        if self.is_empty() {
            self.buf.clear();
            *self.external_cursor = 0;
        } else {
            *self.external_cursor = self.cursor;
        }
    }
}

impl<'a> Read for Drain<'a> {
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

impl<'a> BufRead for Drain<'a> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(self.slice())
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.cursor += amt;
    }
}
