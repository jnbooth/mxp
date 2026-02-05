use std::fmt;

use super::drain::Drain;

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

    #[inline]
    pub fn append<S: AsRef<[u8]>>(&mut self, bytes: S) {
        self.buf.extend_from_slice(bytes.as_ref());
    }

    pub fn drain(&mut self) -> Option<Drain<'_>> {
        if self.buf.is_empty() {
            return None;
        }
        Some(Drain {
            cursor: self.cursor,
            external_cursor: &mut self.cursor,
            buf: &mut self.buf,
        })
    }

    #[inline]
    pub fn write_fmt(&mut self, args: fmt::Arguments) {
        fmt::Write::write_fmt(self, args).unwrap();
    }
}

impl fmt::Write for BufferedInput {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.append(s);
        Ok(())
    }
}
