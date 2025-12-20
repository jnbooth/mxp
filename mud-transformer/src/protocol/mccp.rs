use std::io::{self, BufRead};

use flate2::FlushDecompress;

/// MUD Client Compression Protocol v1
pub const CODE_V1: u8 = 85;

/// MUD Client Compression Protocol v2
pub const CODE_V2: u8 = 86;

#[derive(Debug)]
pub(crate) struct Decompress {
    inner: flate2::Decompress,
    active: bool,
    supports_mccp_2: bool,
}

impl Default for Decompress {
    fn default() -> Self {
        Self::new()
    }
}

impl Decompress {
    pub fn new() -> Self {
        Self {
            inner: flate2::Decompress::new(true),
            active: false,
            supports_mccp_2: false,
        }
    }

    pub const fn active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn will(&mut self, code: u8) -> bool {
        match code {
            CODE_V1 => !self.supports_mccp_2,
            CODE_V2 => {
                self.supports_mccp_2 = true;
                true
            }
            _ => false,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn decompress<R: BufRead>(&mut self, reader: &mut R, buf: &mut [u8]) -> io::Result<usize> {
        let total_in = self.inner.total_in();
        let total_out = self.inner.total_out();
        let status = self
            .inner
            .decompress(reader.fill_buf()?, buf, FlushDecompress::None);

        match status {
            Ok(flate2::Status::Ok) => {
                reader.consume((self.inner.total_in() - total_in) as usize);
                Ok((self.inner.total_out() - total_out) as usize)
            }
            Ok(flate2::Status::BufError) => Ok(0),
            Ok(flate2::Status::StreamEnd) => Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
            Err(e) => Err(e.into()),
        }
    }

    pub fn reset(&mut self) {
        self.inner.reset(true);
    }
}
