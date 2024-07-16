use async_compression::tokio::bufread::ZlibDecoder;
use mud_transformer::{OutputDrain, SideEffect, Transformer, TransformerConfig};
use pin_project_lite::pin_project;
use std::cmp::min;
use std::io::{self, Cursor};
use std::io::{BufRead, IoSlice};
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{
    AsyncBufRead, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf,
};

const COMPRESS_BUFFER: usize = 1024 * 20;
const READ_BUFFER: usize = 1024 * 16; // needs to be <= COMPRESS_BUFFER

pin_project! {
    pub struct MudStream<T> {
        buf: [u8; COMPRESS_BUFFER],
        #[pin]
        stream: DecompressStream<T>,
        transformer: Transformer,
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> MudStream<T> {
    pub fn new(stream: T, config: TransformerConfig) -> Self {
        Self {
            buf: [0; COMPRESS_BUFFER],
            stream: DecompressStream::new(stream),
            transformer: Transformer::new(config),
        }
    }

    pub fn into_inner(self) -> T {
        self.stream.into_inner()
    }

    pub async fn read(&mut self) -> io::Result<Option<OutputDrain>> {
        let n = match self.stream.read(&mut self.buf).await {
            Ok(0) => return Ok(None),
            Ok(n) => n,
            Err(e) => return Err(e),
        };
        let mut iter = self.buf[..n].into_iter();
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

pin_project! {
    struct ZState<R> {
        has_prepend: bool,
        prepend: Cursor<Vec<u8>>,
        #[pin]
        reader: BufReader<R>,
    }
}

impl<R: AsyncRead> ZState<R> {
    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }

    pub fn get_mut(&mut self) -> &mut R {
        self.reader.get_mut()
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for ZState<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if !self.has_prepend {
            return self.project().reader.poll_read(cx, buf);
        }
        let pos = self.prepend.position();
        let slice: &[u8] = self.prepend.get_ref();
        let len = slice.len();
        if pos < len as u64 {
            let start = pos as usize;
            let amt = min(len - start, buf.remaining());
            let end = start + amt;
            buf.put_slice(&slice[start..end]);
            self.prepend.set_position(end as u64);
            if end != len {
                return Poll::Ready(Ok(()));
            }
        }
        self.has_prepend = false;
        Poll::Ready(Ok(()))
    }
}

impl<R: AsyncRead + Unpin> AsyncBufRead for ZState<R> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        let project = self.project();
        if *project.has_prepend {
            return Poll::Ready(project.prepend.fill_buf());
        }
        project.reader.poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        let project = self.project();
        if !*project.has_prepend {
            project.reader.consume(amt);
            return;
        }

        let pos = project.prepend.position();
        let len = project.prepend.get_ref().len();
        let remaining = if pos >= len as u64 {
            *project.has_prepend = false;
            amt
        } else {
            let unused_space = pos as usize - len;
            project.prepend.consume(unused_space);
            amt - unused_space
        };
        project.reader.consume(remaining);
    }
}

pin_project! {
    #[project = DecompressStreamProj]
    enum DecompressStream<R> {
        Uncompressed { #[pin] reader: R },
        Compressed { #[pin] reader: ZlibDecoder<ZState<R>> },
        Transitioning,
    }
}

impl<R: AsyncRead + Unpin> DecompressStream<R> {
    pub fn into_inner(self) -> R {
        match self {
            Self::Uncompressed { reader } => reader,
            Self::Compressed { reader } => reader.into_inner().into_inner(),
            Self::Transitioning => unreachable!(),
        }
    }

    pub const fn new(reader: R) -> Self {
        Self::Uncompressed { reader }
    }

    pub fn get_mut(&mut self) -> &mut R {
        match self {
            Self::Uncompressed { reader } => reader,
            Self::Compressed { reader } => reader.get_mut().get_mut(),
            Self::Transitioning => unreachable!(),
        }
    }

    pub fn reset(&mut self) {
        let mut buf = Self::Transitioning;
        mem::swap(self, &mut buf);
        *self = Self::Uncompressed {
            reader: buf.into_inner(),
        };
    }

    pub fn start_decompressing(&mut self, prepend: Vec<u8>) {
        let mut buf = Self::Transitioning;
        mem::swap(self, &mut buf);
        let reader = buf.into_inner();
        let inner = ZState {
            has_prepend: !prepend.is_empty(),
            prepend: Cursor::new(prepend),
            reader: BufReader::with_capacity(READ_BUFFER, reader),
        };
        *self = Self::Compressed {
            reader: ZlibDecoder::new(inner),
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for DecompressStream<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.project() {
            DecompressStreamProj::Uncompressed { reader } => reader.poll_read(cx, buf),
            DecompressStreamProj::Compressed { reader } => reader.poll_read(cx, buf),
            DecompressStreamProj::Transitioning => unreachable!(),
        }
    }
}
