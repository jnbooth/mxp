use std::fmt;
use std::io::BufRead;

pub use zlib_rs::{Inflate, InflateError, InflateFlush, ReturnCode, Status};

/// MUD Client Compression Protocol v2
///
/// https://tintin.mudhalla.net/protocols/mccp/
pub const OPT: u8 = 86;

/// The state that is used to decompress an input.
pub(crate) struct Decompress {
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
    /// Creates a new decompression state.
    pub fn new() -> Self {
        Self {
            inner: Inflate::new(true, 15),
            active: false,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn decompress(
        &mut self,
        input: &mut &[u8],
        output: &mut [u8],
    ) -> (usize, Result<Status, Error>) {
        let empty = input.is_empty();
        let flush = if empty {
            InflateFlush::SyncFlush
        } else {
            InflateFlush::NoFlush
        };
        let total_in = self.inner.total_in();
        let total_out = self.inner.total_out();
        let result = match self.inner.decompress(input, output, flush) {
            Ok(Status::StreamEnd) => Ok(Status::StreamEnd),
            Ok(Status::Ok) => Ok(Status::Ok),
            Ok(Status::BufError) if empty => Ok(Status::Ok),
            Ok(Status::BufError) => Err(Error::BufError { size: output.len() }),
            Err(InflateError::NeedDict { dict_id }) => Err(Error::NeedDict { dict_id }),
            Err(InflateError::StreamError) => Err(Error::StreamError),
            Err(InflateError::DataError) => Err(Error::DataError),
            Err(InflateError::MemError) => Err(Error::MemError),
        };
        let new_in = (self.inner.total_in() - total_in) as usize;
        let new_out = (self.inner.total_out() - total_out) as usize;
        input.consume(new_in);
        (new_out, result)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum Error {
    /// Decompressing this input requires a dictionary.
    NeedDict { dict_id: u32 } = 2,
    /// The [`Inflate`] is in an inconsistent state, most likely
    /// due to an invalid configuration parameter.
    StreamError = -2,
    /// The input is not a valid deflate stream.
    DataError = -3,
    /// A memory allocation failed.
    MemError = -4,
    /// Decompression buffer is too small.
    BufError { size: usize } = -5,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NeedDict { .. } => f.write_str("need dictionary"),
            Self::StreamError => f.write_str("stream error"),
            Self::DataError => f.write_str("data error"),
            Self::MemError => f.write_str("insufficient memory"),
            Self::BufError { .. } => f.write_str("buffer too small"),
        }
    }
}

impl std::error::Error for Error {}
