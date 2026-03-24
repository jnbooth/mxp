use std::{fmt, io};

use super::drain::InputDrain;

#[derive(Clone, Debug)]
pub(crate) struct BufferedInput {
    buf: Vec<u8>,
    cursor: usize,
}

impl Default for BufferedInput {
    fn default() -> Self {
        Self::new()
    }
}

impl BufferedInput {
    pub const fn new() -> Self {
        Self {
            buf: Vec::new(),
            cursor: 0,
        }
    }

    pub fn drain(&mut self) -> Option<InputDrain<'_>> {
        if self.buf.is_empty() {
            return None;
        }
        Some(InputDrain {
            cursor: self.cursor,
            external_cursor: &mut self.cursor,
            buf: &mut self.buf,
        })
    }

    #[inline]
    pub fn write(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    #[inline]
    pub fn write_str(&mut self, s: &str) {
        self.buf.extend_from_slice(s.as_bytes());
    }

    #[inline]
    pub fn write_fmt(&mut self, args: fmt::Arguments) {
        fmt::Write::write_fmt(self, args).unwrap();
    }
}

impl fmt::Write for BufferedInput {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

impl io::Write for BufferedInput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.buf.write_vectored(bufs)
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.buf.extend_from_slice(buf);
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        self.write_fmt(args);
        Ok(())
    }
}
