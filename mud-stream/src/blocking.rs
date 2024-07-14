use mud_transformer::{OutputFragment, Transformer, TransformerConfig};
use std::io::{self, IoSlice, Read, Write};
use std::net::TcpStream;
use std::vec;

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

    pub fn read(&mut self) -> io::Result<Option<vec::Drain<OutputFragment>>> {
        let n = self.stream.read(&mut self.buf)?;
        if n == 0 {
            return Ok(None);
        }
        let received = &self.buf[..n];
        for &c in received {
            self.transformer.read_byte(c);
        }
        self.transformer
            .drain_input()
            .write_all_to(&mut self.stream)?;
        Ok(Some(self.transformer.drain_output()))
    }
}
impl Write for MudStream {
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
