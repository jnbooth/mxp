use std::fmt;
use std::io::{self, BufRead};

use zlib_rs::{Inflate, InflateError, InflateFlush, Status};

/// MUD Client Compression Protocol v2
///
/// https://tintin.mudhalla.net/protocols/mccp/
pub const OPT: u8 = 86;

pub struct Decompress {
    inner: Inflate,
    active: bool,
}

impl fmt::Debug for Decompress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Decompress")
            .field("active", &self.active)
            .finish_non_exhaustive()
    }
}

impl Default for Decompress {
    fn default() -> Self {
        Self::new()
    }
}

impl Decompress {
    pub fn new() -> Self {
        Self {
            inner: Inflate::new(true, 15),
            active: false,
        }
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn decompress<R: BufRead>(
        &mut self,
        input: &mut R,
        output: &mut [u8],
    ) -> io::Result<usize> {
        let total_in = self.inner.total_in();
        let total_out = self.inner.total_out();
        let status = self
            .inner
            .decompress(input.fill_buf()?, output, InflateFlush::NoFlush);

        match status {
            Ok(Status::Ok) => {}
            Ok(Status::StreamEnd) => {
                self.active = false;
            }
            Ok(Status::BufError) => return Ok(0),
            Err(e) => return Err(io::Error::new(get_error_kind(e), e.as_str())),
        }
        input.consume((self.inner.total_in() - total_in) as usize);
        Ok((self.inner.total_out() - total_out) as usize)
    }

    pub fn reset(&mut self) {
        self.inner.reset(true);
    }
}

fn get_error_kind(error: InflateError) -> io::ErrorKind {
    match error {
        InflateError::DataError => io::ErrorKind::InvalidData,
        InflateError::MemError => io::ErrorKind::OutOfMemory,
        _ => io::ErrorKind::Other,
    }
}
