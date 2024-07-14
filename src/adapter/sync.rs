use crate::style::OutputFragment;
use crate::transformer::{Transformer, TransformerConfig};
use std::io::{self, Read, Write};
use std::net::TcpStream;

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

    pub fn read<W: Write>(&mut self, mut output: W) -> io::Result<usize> {
        let n = self.stream.read(&mut self.buf)?;
        if n == 0 {
            return Ok(n);
        }
        let received = &self.buf[..n];
        for &c in received {
            self.transformer.interpret_char(c);
        }
        for fragment in self.transformer.drain_output() {
            if let OutputFragment::Text(fragment) = fragment {
                output.write_all(fragment.as_ref())?;
            }
        }
        let mut input = self.transformer.drain_input();
        input.write_all_to(&mut self.stream)?;
        Ok(n)
    }
}
