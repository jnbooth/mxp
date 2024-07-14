use std::io::{self, Write};
use std::net::TcpStream;

use mxp::{sync, TransformerConfig};

fn main() -> io::Result<()> {
    let stream = TcpStream::connect(("discworld.atuin.net", 4242))?;
    let mut stream = sync::MudStream::new(stream, TransformerConfig::new());
    let mut stdout = io::stdout().lock();
    while let Some(output) = stream.read()? {
        for fragment in output {
            if let Some(bytes) = fragment.as_bytes() {
                stdout.write_all(bytes)?;
            }
        }
    }

    Ok(())
}
