use crate::config::READ_BUFFER;

use super::decompress::DecompressStream;
use mud_transformer::{OutputDrain, SideEffect, Transformer, TransformerConfig};
use pin_project_lite::pin_project;
use std::io;
use std::io::IoSlice;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pin_project! {
    pub struct MudStream<T> {
        #[pin]
        stream: DecompressStream<T>,
        transformer: Transformer,
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> MudStream<T> {
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

    pub async fn read(&mut self) -> io::Result<Option<OutputDrain>> {
        let mut buf = [0; READ_BUFFER];
        let n = match self.stream.read(&mut buf).await {
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
            self.stream
                .get_mut()
                .write_all_buf(&mut self.transformer.drain_input())
                .await?;
        }
        Ok(Some(self.transformer.drain_output()))
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for MudStream<T> {
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
        Pin::new(self.stream.get_mut()).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
        Pin::new(self.stream.get_mut()).poll_shutdown(cx)
    }

    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(self.stream.get_mut()).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        bufs: &[IoSlice],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(self.stream.get_mut()).poll_write_vectored(cx, bufs)
    }
}
