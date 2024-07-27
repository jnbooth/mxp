use crate::config::{COMPRESS_BUFFER, READ_BUFFER};

use mud_transformer::{OutputDrain, Transformer, TransformerConfig};
use pin_project_lite::pin_project;
use std::io;
use std::io::IoSlice;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pin_project! {
    pub struct MudStream<T> {
        done: bool,
        #[pin]
        stream: T,
        transformer: Transformer,
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> MudStream<T> {
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

    pub async fn read(&mut self) -> io::Result<Option<OutputDrain>> {
        if self.done {
            return Ok(None);
        }

        let mut buf = [0; READ_BUFFER];
        let n = self.stream.read(&mut buf).await?;
        if n == 0 {
            self.done = true;
            return Ok(Some(self.transformer.flush_output()));
        }

        self.transformer
            .receive(&buf[..n], &mut [0; COMPRESS_BUFFER])?;
        if let Some(mut drain) = self.transformer.drain_input() {
            self.stream.write_all_buf(&mut drain).await?;
        }
        Ok(Some(self.transformer.drain_output()))
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for MudStream<T> {
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
        self.project().stream.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
        self.project().stream.poll_shutdown(cx)
    }

    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.project().stream.poll_write(cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context,
        bufs: &[IoSlice],
    ) -> Poll<Result<usize, io::Error>> {
        self.project().stream.poll_write_vectored(cx, bufs)
    }
}
