use crate::style::OutputFragment;
use crate::transformer::{Transformer, TransformerConfig};
use std::io::IoSlice;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{io, vec};
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct MudStream {
    buf: [u8; 1024],
    stream: TcpStream,
    transformer: Transformer,
}

impl MudStream {
    pub fn new(stream: TcpStream, config: TransformerConfig) -> Self {
        Self {
            buf: [0; 1024],
            stream,
            transformer: Transformer::new(config),
        }
    }

    pub fn into_tcp(self) -> TcpStream {
        self.stream
    }

    pub async fn read(&mut self) -> io::Result<Option<vec::Drain<OutputFragment>>> {
        let n = self.stream.read(&mut self.buf).await?;
        if n == 0 {
            return Ok(None);
        }
        let received = &self.buf[..n];
        for &c in received {
            self.transformer.interpret_char(c);
        }
        self.stream
            .write_all_buf(&mut self.transformer.drain_input())
            .await?;
        Ok(Some(self.transformer.drain_output()))
    }
}
impl AsyncWrite for MudStream {
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
