use std::io;
use std::io::IoSlice;
use std::pin::Pin;
use std::task::{Context, Poll};

use mud_transformer::{OutputDrain, Transformer, TransformerConfig};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::config::DEFAULT_BUFFER_SIZE;

pub struct MudStream<T> {
    done: bool,
    stream: T,
    transformer: Transformer,
    buf: Vec<u8>,
    midpoint: usize,
}

impl<T: AsyncRead + AsyncWrite + Unpin> MudStream<T> {
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

    pub fn set_config(&mut self, config: TransformerConfig) {
        self.transformer.set_config(config);
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

    pub async fn read(&mut self) -> io::Result<Option<OutputDrain>> {
        if self.done {
            return Ok(None);
        }

        let n = self.stream.read(&mut self.buf[..self.midpoint]).await?;
        if n == 0 {
            self.done = true;
            return Ok(Some(self.transformer.flush_output()));
        }

        let (received, decompress_buf) = self.buf.split_at_mut(n);
        self.transformer.receive(received, decompress_buf)?;

        if let Some(mut drain) = self.transformer.drain_input() {
            self.stream.write_all_buf(&mut drain).await?;
        }
        Ok(Some(self.transformer.drain_output()))
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for MudStream<T> {
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }

    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        bufs: &[IoSlice],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.stream).poll_write_vectored(cx, bufs)
    }
}
