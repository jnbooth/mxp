use crate::config::{COMPRESS_BUFFER, READ_BUFFER};

use mud_transformer::{OutputDrain, Transformer, TransformerConfig};
use std::io::{self, IoSlice, Read, Write};

#[derive(Debug)]
pub struct MudStream<T> {
    done: bool,
    stream: T,
    transformer: Transformer,
}

impl<T: Read + Write> MudStream<T> {
    pub fn new(stream: T, config: TransformerConfig) -> Self {
        Self {
            done: false,
            stream,
            transformer: Transformer::new(config),
        }
    }

    pub fn into_inner(self) -> T {
        self.stream
    }

    pub fn into_transformer(self) -> Transformer {
        self.transformer
    }

    pub fn into_pair(self) -> (T, Transformer) {
        (self.stream, self.transformer)
    }

    pub fn get_ref(&self) -> &T {
        &self.stream
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.stream
    }

    pub fn read(&mut self) -> io::Result<Option<OutputDrain>> {
        if self.done {
            return Ok(None);
        }

        let mut buf = [0; READ_BUFFER];
        let n = self.stream.read(&mut buf)?;
        if n == 0 {
            self.done = true;
            return Ok(Some(self.transformer.flush_output()));
        }

        self.transformer
            .receive(&buf[..n], &mut [0; COMPRESS_BUFFER])?;
        self.transformer
            .drain_input()
            .write_all_to(&mut self.stream)?;
        Ok(Some(self.transformer.drain_output()))
    }
}
impl<T: Write> Write for MudStream<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice]) -> io::Result<usize> {
        self.stream.write_vectored(bufs)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}
