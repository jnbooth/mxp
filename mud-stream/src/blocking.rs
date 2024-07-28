use mud_transformer::{OutputDrain, Transformer, TransformerConfig};
use std::io::{self, IoSlice, Read, Write};

use crate::config::DEFAULT_BUFFER_SIZE;

#[derive(Debug)]
pub struct MudStream<T> {
    done: bool,
    stream: T,
    transformer: Transformer,
    buf: Vec<u8>,
    midpoint: usize,
}

impl<T: Read + Write> MudStream<T> {
    pub fn new(stream: T, config: TransformerConfig) -> Self {
        Self::with_capacity(stream, config, DEFAULT_BUFFER_SIZE)
    }

    pub fn with_capacity(stream: T, config: TransformerConfig, capacity: usize) -> Self {
        Self {
            done: false,
            stream,
            transformer: Transformer::new(config),
            buf: vec![0; capacity],
            midpoint: capacity / 2,
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

        let n = self.stream.read(&mut self.buf[..self.midpoint])?;
        if n == 0 {
            self.done = true;
            return Ok(Some(self.transformer.flush_output()));
        }

        let (received, decompress_buf) = self.buf.split_at_mut(n);

        self.transformer.receive(received, decompress_buf)?;
        if let Some(mut drain) = self.transformer.drain_input() {
            drain.write_all_to(&mut self.stream)?
        }
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
