use std::io;
use std::net::TcpStream;

use mxp::{sync, TransformerConfig};

fn main() -> io::Result<()> {
    let stream = TcpStream::connect(("discworld.atuin.net", 4242))?;
    let mut stream = sync::MudStream::new(stream, TransformerConfig::new());
    let mut stdout = io::stdout().lock();
    loop {
        if stream.read(&mut stdout)? == 0 {
            return Ok(());
        }
    }
}
