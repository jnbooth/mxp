use std::io::{self, BufRead};

use flate2::FlushDecompress;

#[derive(Debug)]
pub struct Decompress {
    inner: flate2::Decompress,
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
        }
    }

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
