use crate::config::READ_BUFFER;

use super::decompress::DecompressStream;
use mud_transformer::{OutputDrain, SideEffect, Transformer, TransformerConfig};
use std::io::{self, IoSlice, Read, Write};

#[derive(Debug)]
pub struct MudStream<T> {
    stream: DecompressStream<T>,
    transformer: Transformer,
}

impl<T: Read + Write> MudStream<T> {
    pub fn new(stream: T, config: TransformerConfig) -> Self {
        Self {
            stream: DecompressStream::new(stream),
            transformer: Transformer::new(config),
        }
    }

    pub fn into_inner(self) -> T {
        self.stream.into_inner()
    }

    pub fn get_ref(&self) -> &T {
        self.stream.get_ref()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.stream.get_mut()
    }

    pub fn read(&mut self) -> io::Result<Option<OutputDrain>> {
        let mut buf = [0; READ_BUFFER];
        let n = match self.stream.read(&mut buf) {
            Ok(0) => return Ok(None),
            Ok(n) => n,
            Err(e) => return Err(e),
        };
        let mut iter = buf[..n].into_iter();
        while let Some(&c) = iter.next() {
            match self.transformer.read_byte(c) {
                Some(SideEffect::DisableCompression) => self.stream.reset(),
                Some(SideEffect::EnableCompression) => {
                    let remaining: Vec<u8> = iter.as_slice().to_vec();
                    iter.nth(remaining.len()); // advance to end
                    self.stream.start_decompressing(remaining);
                }
                _ => (),
            }
            self.transformer
                .drain_input()
                .write_all_to(&mut self.stream.get_mut())?;
        }
        Ok(Some(self.transformer.drain_output()))
    }
}
impl<T: Write> Write for MudStream<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.get_mut().write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice]) -> io::Result<usize> {
        self.stream.get_mut().write_vectored(bufs)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.stream.get_mut().flush()
    }
}
