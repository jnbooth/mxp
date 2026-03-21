use std::fmt;

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
    pub fn write_fmt(&mut self, args: fmt::Arguments) {
        fmt::Write::write_fmt(self, args).unwrap();
    }
}

impl fmt::Write for BufferedInput {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buf.extend_from_slice(s.as_bytes());
        Ok(())
    }
}
